---
layout: post
title: "Managing a Pool of extern \"C\" fns"
date: 2024-08-30
---

# Managing a Pool of `extern "C" fn`s

This article continues on from [the previous one](016-rust-cpp-proc-macro.md) which details how we wrote the procedural macro used to generate the `extern "C" fn`s used as callbacks we pass to the C++ [vsomeip](https://github.com/COVESA/vsomeip) library. In [Eclipse uProtocol](https://github.com/eclipse-uprotocol) we are building on top of vsomeip in order to enable communication over SOME/IP to mechatronics devices (think e.g. brake controllers or IMUs).

Here we'll go through the implementation of managing the pools of `extern "C" fn`s.

## Small bit of the proc macro

I'll need to bring in a [bit more](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/vsomeip-proc-macro/src/lib.rs#L30) of the proc macro here that lets us generate a book-keeping mechanism to ensure `extern "C" fn`s are used at most once.

I'll leave some `PELE` comments below expanding on some points.

```rust
extern crate proc_macro;


use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, LitInt};

#[proc_macro]
pub fn generate_message_handler_extern_c_fns(input: TokenStream) -> TokenStream {
    let num_fns = parse_macro_input!(input as LitInt)
        .base10_parse::<usize>()
        .unwrap();

    // ... snip ...

    // PELE: Making a quote!{} block here that initializes a HashSet
    //   with the #num_fns as a literal
    let mut message_handler_ids_init = quote! {
        let mut set = HashSet::with_capacity(#num_fns);
    };

    for i in 0..num_fns {
        // ... snip ...

        // PELE: We insert each #i message_handler_id for this
        //   extern "C" fn we're creating above in the // ... snip ... section
        message_handler_ids_init.extend(quote! {
            set.insert(#i);
        });
    }

    let expanded = quote! {
        // PELE: Here we use the pub(super) Rust visibility specifier
        //   to allow the outer module in the rust visible to the outer
        //   module, i.e. the module in which we call the proc macro
        pub(super) mod message_handler_proc_macro {
            // PELE: Here we're pulling in everything from the outer module,
            //   i.e. the module in which we call the proc macro
            use super::*;

            // PELE: We use a lazy_static!{} block here to make a static
            //   ref to a RwLock<HashSet<usize>>
            //   In other words, we want a HashSet which will hold all the
            //   message_handler_ids and then insert one per extern "C" fn
            //   we generated (the #message_handler_ids_init bit)
            lazy_static! {
                pub(super) static ref FREE_MESSAGE_HANDLER_IDS: RwLock<HashSet<usize>> = {
                    #message_handler_ids_init
                    RwLock::new(set)
                };
            }

            // ... snip ...

            // PELE: We use this function from the outer module in order to obtain
            //   an extern "C" fn with the proper signature to be used by
            //   the vsomeip library's application::register_message_handler() function
            pub(super) fn get_extern_fn(message_handler_id: usize) -> extern "C" fn(&SharedPtr<vsomeip::message>) {
                trace!("get_extern_fn with message_handler_id: {}", message_handler_id);
                match message_handler_id{
                    #(#match_arms)*
                    _ => panic!("MessageHandlerId out of range: {message_handler_id}"),
                }
            }
        }
    }
}
```

## Managing the pool of `extern "C" fn`s

We have a pool of these `extern "C" fn`s that we want to ensure are used _at most once_ for the duration they are registered.

I concentrated the mechanisms for this within [`message_handler_registry.rs`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/main/up-transport-vsomeip/src/storage/message_handler_registry.rs). I'll walk through the key portions:

* the [`ProcMacroMessageHandlerAccess`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L51) shim by which the proc macro can access the [`InMemoryMessageHandlerRegistry`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L153)
* the [`InMemoryMessageHandlerRegistry`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L153) by which we can then retrieve new `MessageHandlerFnPtr` to feed into the vsomeip library

In the next section we'll talk through both bullet points above, in reverse order.

## `InMemoryMessageHandlerRegistry`

The [`InMemoryMessageHandlerRegistry`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L149-L157) struct looks like this.

`MessageHandlerIdAndListenerConfig` is a [`BiMap`](https://docs.rs/bimap/0.6.3/bimap/type.BiMap.html) (i.e. a bijective map) from the [`bimap`](https://crates.io/crates/bimap) crate. Because the `MessageHandlerId` and tuple containing the `(UUri, Option<UUri>, ComparableListener)` have a unique one-to-one mapping.

The other two `HashMap`s map from/to the `MessageHandlerId` to/from the `ClientId` (which identifies the vsomeip `application` this `MessageHandlerId` belongs to).

```rust
type MessageHandlerIdAndListenerConfig =
    BiMap<MessageHandlerId, (UUri, Option<UUri>, ComparableListener)>;
type MessageHandlerIdToClientId = HashMap<MessageHandlerId, ClientId>;
type ClientIdToMessageHandlerId = HashMap<ClientId, HashSet<MessageHandlerId>>;
pub struct InMemoryMessageHandlerRegistry {
    message_handler_id_and_listener_config: RwLock<MessageHandlerIdAndListenerConfig>,
    message_handler_id_to_client_id: RwLock<MessageHandlerIdToClientId>,
    client_id_to_message_handler_id: RwLock<ClientIdToMessageHandlerId>,
}
```

### `new()`

The [`new()`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L160) associated function is straightforward.

```rust
impl InMemoryMessageHandlerRegistry {
    pub fn new() -> Self {
        Self {
            message_handler_id_and_listener_config: RwLock::new(BiMap::new()),
            message_handler_id_to_client_id: RwLock::new(HashMap::new()),
            client_id_to_message_handler_id: RwLock::new(HashMap::new()),
        }
    }
    
    // ... snip ...
}
```

### `get_message_handler()`

The [`get_message_handler()`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L169) function is where we ensure that we always retrieve a [`MessageHandlerFnPtr`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/vsomeip-sys/src/extern_callback_wrappers.rs#L45) which is not in use and, if there's some issue while obtaining one we will roll back any book-keeping we have already completed.

```rust
/// A Rust wrapper around the extern "C" fn used when registering a [message_handler_t](crate::vsomeip::message_handler_t)
///
/// # Rationale
///
/// We want the ability to think at a higher level and not need to consider the underlying
/// extern "C" fn, so we wrap this here in a Rust struct
#[repr(transparent)]
#[derive(Debug)]
pub struct MessageHandlerFnPtr(pub extern "C" fn(&SharedPtr<vsomeip::message>));
```

Let's dig into `get_message_handler()`. I'll highlight some areas of interest with `PELE` comments.

```rust
impl InMemoryMessageHandlerRegistry {
    // ... snip ...

    /// Gets an unused [MessageHandlerFnPtr] to hand over to a vsomeip application
    pub fn get_message_handler(
        &self,
        client_id: ClientId,
        transport_storage: Arc<UPTransportVsomeipStorage>,
        listener_config: (UUri, Option<UUri>, ComparableListener),
    ) -> Result<MessageHandlerFnPtr, GetMessageHandlerError> {
        // Lock all the necessary state at the beginning so we don't have partial transactions
        let mut message_handler_id_to_transport_storage =
            MESSAGE_HANDLER_ID_TO_TRANSPORT_STORAGE.write().unwrap();
        // PELE: Here we are obtaining a write lock on the FREE_MESSAGE_HANDLER_IDS
        //   that's been generated as a part of the proc macro
        let mut free_message_handler_ids = message_handler_proc_macro::FREE_MESSAGE_HANDLER_IDS
            .write()
            .unwrap();

        // PELE: Obtaining write lock on all of our internally held state
        let mut message_handler_id_and_listener_config =
            self.message_handler_id_and_listener_config.write().unwrap();
        let mut message_handler_id_to_client_id =
            self.message_handler_id_to_client_id.write().unwrap();
        let mut client_id_to_message_handler_id =
            self.client_id_to_message_handler_id.write().unwrap();

        let (source_filter, sink_filter, comparable_listener) = listener_config;

        let Ok(message_handler_id) =
            // PELE: Here we are attempting to obtain a free message_handler_id
            Self::find_available_message_handler_id(free_message_handler_ids.deref_mut())
        else {
            return Err(GetMessageHandlerError::OtherError(format!(
                "{:?}",
                UStatus::fail_with_code(UCode::RESOURCE_EXHAUSTED, "No more available extern fns",)
            )));
        };

        // PELE: Here we are using a Vec<Box<dyn FnOnce() + `a>>
        //   to hold the earlier steps we should roll back, should
        //   a later step fail
        type RollbackSteps<'a> = Vec<Box<dyn FnOnce() + 'a>>;
        let mut rollback_steps: RollbackSteps = Vec::new();

        // PELE: As an example of the above, here we are pushing
        //   a call to Self::free_message_handler_id(), the dual and
        //   opposite of Self::find_available_message_handler_id
        //   which will free the MessageHandlerId
        rollback_steps.push(Box::new(move || {
            if let Err(warn) = Self::free_message_handler_id(
                free_message_handler_ids.deref_mut(),
                message_handler_id,
            ) {
                warn!("rolling back: free_message_handler_id: {warn}");
            }
        }));

        // PELE: Here we attempt to insert the message_handler_id and
        //   transport...
        let insert_res = Self::insert_message_handler_id_transport(
            message_handler_id_to_transport_storage.deref_mut(),
            message_handler_id,
            transport_storage,
        );
        // PELE: and if we fail to do so, we will simply go through the
        //   rollback_steps and call each one to make sure this entire
        //   function atomic
        if let Err(err) = insert_res {
            for rollback_step in rollback_steps {
                rollback_step();
            }

            return Err(GetMessageHandlerError::OtherError(format!("{:?}", err)));
        }

        // PELE: You might get the flavor now -- Self::remove_message_handler_id_transport
        //   is the dual and opposite of Self::insert_message_handler_id_transport.
        //   So we now push a call to it into rollback_steps
        rollback_steps.push(Box::new(move || {
            if let Err(warn) = Self::remove_message_handler_id_transport(
                message_handler_id_to_transport_storage.deref_mut(),
                message_handler_id,
            ) {
                warn!("rolling back: remove_listener_id_transport: {warn}");
            }
        }));

        // PELE: Similar logic follows below, will elide most further
        //   explanation

        let insert_res = Self::insert_message_handler_id_client_id(
            message_handler_id_to_client_id.deref_mut(),
            client_id_to_message_handler_id.deref_mut(),
            message_handler_id,
            client_id,
        );
        if let Some(previous_entry) = insert_res {
            let message_handler_id = previous_entry.0;
            let client_id = previous_entry.1;

            for rollback_step in rollback_steps {
                rollback_step();
            }

            return Err(GetMessageHandlerError::OtherError(
                format!("{:?}", UStatus::fail_with_code(
                    UCode::ALREADY_EXISTS, format!(
                        "We already had used that listener_id with a client_id. listener_id: {} client_id: {}",
                        message_handler_id, client_id))
                )
            )
            );
        }

        let listener_config = (
            source_filter.clone(),
            sink_filter.clone(),
            comparable_listener.clone(),
        );

        rollback_steps.push(Box::new(move || {
            if Self::remove_client_id_based_on_message_handler_id(
                message_handler_id_to_client_id.deref_mut(),
                client_id_to_message_handler_id.deref_mut(),
                message_handler_id,
            )
            .is_none()
            {
                warn!("No client_id found to remove for message_handler_id: {message_handler_id}");
            }
        }));

        let insert_res = Self::insert_message_handler_id_and_listener_config(
            message_handler_id_and_listener_config.deref_mut(),
            message_handler_id,
            listener_config,
        );
        if let Err(err) = insert_res {
            for rollback_step in rollback_steps {
                rollback_step();
            }
            return Err(match err {
                MessageHandlerIdAndListenerConfigError::MessageHandlerIdAlreadyExists(
                    listener_id,
                ) => GetMessageHandlerError::ListenerIdAlreadyExists(listener_id),
                MessageHandlerIdAndListenerConfigError::ListenerConfigAlreadyExists => {
                    GetMessageHandlerError::ListenerConfigAlreadyExists(MessageHandlerFnPtr(
                        message_handler_proc_macro::get_extern_fn(message_handler_id),
                    ))
                }
            });
        }

        rollback_steps.push(Box::new(move || {
            if let Err(warn) = Self::remove_message_handler_id_and_listener_config_based_on_message_handler_id(message_handler_id_and_listener_config.deref_mut(), message_handler_id)
            {
                warn!("rolling back: remove_listener_id_and_listener_config_based_on_listener_id: {warn}");
            }
        }));

        // PELE: Finally down here if we're in a good state we can call the
        //   get_extern_fn() shown above and wrap it into a MessageHandlerFnPtr
        Ok(MessageHandlerFnPtr(
            message_handler_proc_macro::get_extern_fn(message_handler_id),
        ))
    }
    // ... snip ...
}
```

There is a [`release_message_handler()`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L311) which, well, releases a message handler. It has the following signature.

```rust
    /// Release a given message handler
    pub fn release_message_handler(
        &self,
        listener_config: (UUri, Option<UUri>, ComparableListener),
    ) -> Result<ClientUsage, UStatus>
```

I won't delve into the details here, but you can imagine it's essentially doing all the rollback_steps as up above in order to ensure our book-keeping is refreshed to not include usage of the `MessageHandlerFnPtr` corresponding to the `listener_config`.

## `ProcMacroMessageHandlerAccess`

The [`ProcMacroMessageHandlerAccess`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L51) struct is essentially an empty facade through which a function generated by the `generate_message_handler_extern_c_fns()` can interact with the `InMemoryMessageHandlerRegistry`.

I like this approach because we're not directly exposing the `lazy_static!` variable that's holding the book-keeping of `MessageHandlerId` -> `std::sync::Weak<UPTransportVsomeipStorage>`, but instead able to have more careful control over the internal semantics of that book-keeping.

### The (loosely held) state

The `lazy_static` [`MESSAGE_HANDLER_ID_TO_TRANSPORT_STORAGE`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L37-L47) is used to maintain a mapping between `MessageHandlerId` -> `std::sync::Weak<UPTransportVsomeipStorage>` so that any individual `extern "C" fn` which has the `MessageHandlerId` baked into it will be able to obtain the necessary state related to the `UPTransportVsomeip` needed to call the associated `UListener`.

I should note here that a [`std::sync::Weak`](https://doc.rust-lang.org/stable/std/sync/struct.Weak.html) pointer type is used very intentionally here in order to ensure that we're incrementing the reference counter of the [`std::sync::Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html) holding the `UPTransportVsomeipStorage`. By doing so, we ensure that the shutdown process and freeing of vsomeip-related resources is done cleaner. I'll probably flesh this out another day.

```rust
type MessageHandlerIdToTransportStorage =
    HashMap<MessageHandlerId, Weak<UPTransportVsomeipStorage>>;
lazy_static! {
    /// A mapping from extern "C" fn [MessageHandlerId] onto [std::sync::Weak] references to [UPTransportVsomeipStorage]
    ///
    /// Used within the context of the proc macro crate (vsomeip-proc-macro) generated [call_shared_extern_fn]
    /// to obtain the state of the transport needed to perform ingestion of vsomeip messages from
    /// within callback functions registered with vsomeip
    static ref MESSAGE_HANDLER_ID_TO_TRANSPORT_STORAGE: RwLock<MessageHandlerIdToTransportStorage> =
        RwLock::new(HashMap::new());
}
```

### The facade

The [`ProcMacroMessageHandlerAccess`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L49-L51) facade struct through which we have a function that interacts with `MESSAGE_HANDLER_ID_TO_TRANSPORT_STORAGE` is, well, a facade. Nothing really going on.

```rust
/// A facade struct from which the proc macro crate (vsomeip-proc-macro) generated `call_shared_extern_fn`
/// can access state related to the transport
struct ProcMacroMessageHandlerAccess;
```

### How the proc macro uses the facade

[Here](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L53-L68) we implement an associated function `get_message_handler_id_transport()` on `ProcMacroMessageHandlerAccess`.

I've included some `PELE` comments below describing what's happening.

```rust
impl ProcMacroMessageHandlerAccess {
    /// Gets a trait object holding transport storage
    ///
    /// # Parameters
    ///
    /// * `message_handler_id`
    fn get_message_handler_id_transport(
        message_handle_id: MessageHandlerId,
    ) -> Option<Arc<UPTransportVsomeipStorage>> {
        // PELE: We obtain a _read_ lock of the RwLock
        //   MESSAGE_HANDLER_ID_TO_TRANSPORT_STORAGE
        //   since we will not be performing mutation
        let message_handler_id_transport_storage =
            MESSAGE_HANDLER_ID_TO_TRANSPORT_STORAGE.read().unwrap();
        // PELE: Obtain the Weak<UPTransportVsomeipStorage> if it exists,
        //   otherwise return the failure path of None using the ? operator
        let transport = message_handler_id_transport_storage.get(&message_handle_id)?;

        // PELE: Here we either succeed in the call to upgrade() and return
        //   Some(Arc<UPTransportVsomeipStorage>) _or_ the upgrade() fails
        //   due to no other references to the Arc<UPTransportVsomeipStorage>,
        //   in which case we return None
        transport.upgrade()
    }
}
```

[Here](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/vsomeip-proc-macro/src/lib.rs#L77-L87)'s a small snippet of the usage of `get_message_handler_id_transport()` from within the proc macro generated function with `PELE` comments describing some of the details.

```rust
#[proc_macro]
pub fn generate_message_handler_extern_c_fns(input: TokenStream) -> TokenStream {
    // ... snip ...
    let expanded = quote! {
        pub(super) mod message_handler_proc_macro {
        // ... snip ...
            // PELE: This is the function that every generated function calls
            fn call_shared_extern_fn(message_handler_id: usize, vsomeip_msg: &SharedPtr<vsomeip::message>) {
                // PELE: If we find that there is a transport_storage associated with
                //   the message_handler_id we proceed, otherwise we bail out with return
                let transport_storage_res = ProcMacroMessageHandlerAccess::get_message_handler_id_transport(message_handler_id);
                let transport_storage = {
                    match transport_storage_res {
                        Some(transport_storage) => transport_storage.clone(),
                        None => {
                            warn!("No transport storage found for message_handler_id: {message_handler_id}");
                            return;
                        }
                    }
                };
                // ... snip ...
            }
        }
    }
}
```

## And it works!

With this formulation, I was able to continue to build upon vsomeip and the `vsomeip-sys` crate which wraps it to enable the [SOME/IP uP-L1 Transport spec](https://github.com/eclipse-uprotocol/up-spec/blob/main/up-l1/someip.adoc) implementation.

Here we combined the power and readability of Rust's procedural macros with the ability to maintain a pool of the generated `extern "C" fn`s through a book-keeping mechanism.

We have clearly separated off the procedural macro into its own module. We allow interaction between the proc macro and the `InMemoryMessageHandlerRegistry` only through well-defined means, avoiding direct access of `lazy_static` variables.

Thanks for reading!

