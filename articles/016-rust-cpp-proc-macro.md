---
layout: post
title: "Creating a Pool of Callbacks from Rust to C++ with Proc Macros"
date: 2024-08-30
---

# Creating a Pool of Callbacks from Rust to C++ with Proc Macros

In [Eclipse uProtocol](https://github.com/eclipse-uprotocol) we had a need to build on top of the [vsomeip](https://github.com/COVESA/vsomeip) C++ library in order to communicate between high-compute devices and mechatronics devices (think e.g. brake controllers, IMUs). I went over some of the high level details in [this article](002-interop-cpp-rust.md).

In this article I'll delve a little deeper into how I arrived at the design I did for the managed pool of `extern "C" fn`s we we maintain on the Rust side that we feed into the vsomeip library as callbacks.

## The vsomeip API

The vsomeip API we're trying to call is [`application::register_message_handler()`](https://github.com/COVESA/vsomeip/blob/48bc6b8e439371b45bbcfbd5ec94b2e0c37bc2a8/interface/vsomeip/application.hpp#L519-L521).

```cpp
    virtual void register_message_handler(service_t _service,
            instance_t _instance, method_t _method,
            const message_handler_t &_handler) = 0;
```

One of the issues I highlighted in the previous article was the bridging that needed to be done within a Rust and C++ glue layer to turn an `extern "C" fn` pointer into a [`message_handler_t`](https://github.com/COVESA/vsomeip/blob/48bc6b8e439371b45bbcfbd5ec94b2e0c37bc2a8/interface/vsomeip/handler.hpp#L22).

```cpp
typedef std::function< void (const std::shared_ptr< message > &) > message_handler_t;
```

For further details on the how and why of doing this bridging see the [previous article](002-interop-cpp-rust.md).

The thing is, in Rust, we can't (normally) just create `extern "C" fn`s at runtime. From what I can tell, there are some [workarounds](https://www.reddit.com/r/rust/comments/ksfk4j/is_it_possible_to_generate_an_extern_c_function/) which use `unsafe` to do some interesting coercions.

## Proc macros to generate callback functions

Rust is a fairly complex language, with many language features to support many different domains and styles of development. I like to keep things as simple as I can given the requirements and constraints I'm under.

In this case, I decided to use a Rust [proc macro](https://doc.rust-lang.org/reference/procedural-macros.html) to _generate_ a pool of callback functions. I'd then put some book-keeping around these pool of callback functions to ensure that the same callback function would be used no more than once at a time and released correctly back into the pool if unregistered.

A reason I like this approach is that at the end of the day, the code generation code is still well, _mostly just code itself_. I tried to use a [declarative macro](https://doc.rust-lang.org/book/ch19-06-macros.html) first, but found that the (very) different syntax of declarative macros ultimately made correlating bugs back to the macro more challenging.

## Initial prototyping

Rust's great tooling lets us quickly spin up prototypes to test things out. I like to do this in cases where I'm being fairly experimental. In this case I spun up [extern_fn_generator](https://github.com/PLeVasseur/extern_fn_generator) and [test_extern_fn_generator](https://github.com/PLeVasseur/test_extern_fn_generator) to prototype.

I'll walk through a couple of phases of the prototyping to highlight a lesson learned. Spoilers: expanding proc macros can be very time and compute intensive in the compile step, so try to minimize the amount of macro expansion as much as possible!

### A workable, but rough start

After a few initial iterations, I had the `extern_fn_generator` crate working! Problem was -- it was horrible slow. I don't have benchmark numbers handy anymore, but from what I recall compilation was taking somewhere on the order of ten minutes or so for generating ~1000 functions. And realistically, we were looking at somewhere between 1000 to 5000 callback functions happening here, as we'd need to be able to support communication with a wide range of mechatronics devices.

I became worried, as this was just plain unworkable from a developer experience perspective. How in the world could I in good conscience ask anyone to contribute to a codebase where each compile took ten minutes or more?

### Diving in

Let's dig into [`generate_extern_fns()`](https://github.com/PLeVasseur/extern_fn_generator/blob/88846db0f1e72c98593a23cf55b0acffb038701c/src/lib.rs#L8-L58) to see what it looks like.

I'll leave some `PELE` comments to walk through what's going on.

```rust
extern crate proc_macro;


use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{LitInt, parse_macro_input};


#[proc_macro]
pub fn generate_extern_fns(input: TokenStream) -> TokenStream {
    // PELE: We consume a single usize literal in the input and parse it here
    let num_fns = parse_macro_input!(input as LitInt).base10_parse::<usize>().unwrap();

    // PELE: We will use the quote! macro to build up the generated functions in a block
    let mut generated_fns = quote! {};
    // PELE: We will use the quote! macro to build up the match arms to find which
    //   function to call
    let mut match_arms = quote! {};

    // PELE: For the number of functions we were ask to do so
    for i in 0..num_fns {
        // PELE: Create identifiers with the number baked in
        let extern_fn_name = format_ident!("extern_on_msg_wrapper_{}", i);
        let fn_name = format_ident!("on_msg_wrapper_{}", i);

        // PELE: Generate the extern "C" fn for i
        let fn_code = quote! {
            #[no_mangle]
            pub extern "C" fn #extern_fn_name(param: u32) {
                println!("Calling extern function #{}", #i);
                // PELE: Obtain the listener ffrom the registry
                let registry = LISTENER_REGISTRY.lock().unwrap();
                // PELE: If there's a listener at this slot
                if let Some(listener) = registry.get(&#i) {
                    // PELE: Clone the listener
                    let listener = Arc::clone(listener);
                    // PELE: and spawn a new task to run the listener in the
                    //   #fn_name function
                    tokio::spawn(async move {
                        #fn_name(listener, param).await;
                    });
                } else {
                    println!("Listener not found for ID {}", #i);
                }
            }

            // PELE: The function which will call the UListener's on_msg function
            async fn #fn_name(listener: Arc<dyn UListener>, param: u32) {
                listener.on_msg(param).await;
            }
        };

        // PELE: Add the fn_code quote!{} to generated_fns
        generated_fns.extend(fn_code);

        // PELE: Add the match arm to our match_arms
        let match_arm = quote! {
            #i => #extern_fn_name,
        };
        match_arms.extend(match_arm);
    }

    // PELE: We then make an expanded quote!{} block which contains
    let expanded = quote! {
        // PELE: our generated functions
        #generated_fns

        // PELE: and a function to look up the appropriate extern "C" fn
        fn get_extern_fn(listener_id: usize) -> extern "C" fn(u32) {
            match listener_id {
                // PELE: based on the #match_arms
                #match_arms
                _ => panic!("Listener ID out of range"),
            }
        }
    };

    expanded.into()
}
```

### Compilation performance investigation

I dove into the Rust Compiler Development Guide's section on [Performance testing](https://rustc-dev-guide.rust-lang.org/tests/perf.html#performance-testing) and other related resources.

Unfortunately I no longer have these graphs, but signs did point to the macro generation step as being the longest running step by far.

I found this [answer](https://users.rust-lang.org/t/5-hours-to-compile-macro-what-can-i-do/36508/2) by [dtolnay](https://github.com/dtolnay) about compilation with a macro taking 5 (!) hours.

I found his suggestion enlightening:

> LLVM is not good at big functions, I have seen this before as well. One of the functions you are generating is almost 100,000 lines of Rust code including tons of internal control flow: starting here 357, repeated 300× in the definition of VANILLA_ID_MAP. You should reorganize define_blocks to break this one function up into 300 individual functions and it will compile in minutes.

So I set out to improve how I structured the macro.

## New and improved (faster!)

Here's the [new macro](https://github.com/PLeVasseur/extern_fn_generator/blob/2c5f54f77ff15aacfd58e37fd7b9ed345f827d93/src/lib.rs#L8), see if you can spot the difference!

```rust
extern crate proc_macro;


use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{LitInt, parse_macro_input};


#[proc_macro]
pub fn generate_extern_fns(input: TokenStream) -> TokenStream {
    let num_fns = parse_macro_input!(input as LitInt).base10_parse::<usize>().unwrap();


    let mut generated_fns = quote! {};
    let mut match_arms = Vec::with_capacity(num_fns);


    for i in 0..num_fns {
        let extern_fn_name = format_ident!("extern_on_msg_wrapper_{}", i);


        let fn_code = quote! {
            #[no_mangle]
            pub extern "C" fn #extern_fn_name(param: u32) {
                call_shared_extern_fn(#i, param);
            }
        };


        generated_fns.extend(fn_code);


        let match_arm = quote! {
            #i => #extern_fn_name,
        };
        match_arms.push(match_arm);
    }


    let expanded = quote! {
        #generated_fns


        fn call_shared_extern_fn(listener_id: usize, param: u32) {
            println!("Calling extern function #{}", listener_id);
            let registry = LISTENER_REGISTRY.lock().unwrap();
            if let Some(listener) = registry.get(&listener_id) {
                let listener = Arc::clone(listener);
                tokio::spawn(async move {
                    shared_async_fn(listener, param).await;
                });
            } else {
                println!("Listener not found for ID {}", listener_id);
            }
        }


        async fn shared_async_fn(listener: Arc<dyn UListener>, param: u32) {
            listener.on_msg(param).await;
        }


        fn get_extern_fn(listener_id: usize) -> extern "C" fn(u32) {
            match listener_id {
                #(#match_arms)*
                _ => panic!("Listener ID out of range"),
            }
        }
    };


    expanded.into()
}
```

Yes, indeed, the change was drastically reducing the amount of code contained within the `quote!{}` block that we're generating thousands of.

### Digging into the change

Here's the key change:

[Before](https://github.com/PLeVasseur/extern_fn_generator/blob/88846db0f1e72c98593a23cf55b0acffb038701c/src/lib.rs#L8-L58):

```rust
        let fn_code = quote! {
            #[no_mangle]
            pub extern "C" fn #extern_fn_name(param: u32) {
                println!("Calling extern function #{}", #i);
                let registry = LISTENER_REGISTRY.lock().unwrap();
                if let Some(listener) = registry.get(&#i) {
                    let listener = Arc::clone(listener);
                    tokio::spawn(async move {
                        #fn_name(listener, param).await;
                    });
                } else {
                    println!("Listener not found for ID {}", #i);
                }
            }
```

[After](https://github.com/PLeVasseur/extern_fn_generator/blob/2c5f54f77ff15aacfd58e37fd7b9ed345f827d93/src/lib.rs#L17-L22):

```rust
        let fn_code = quote! {
            #[no_mangle]
            pub extern "C" fn #extern_fn_name(param: u32) {
                call_shared_extern_fn(#i, param);
            }
        };
```

Note the major changes here to reduce macro expansion time. Calling a _single_ function with a parameter within the body allows the Rust compiler to avoid having to do any of a bunch of things.

* removal of interactions with any variables, so no scope checking
* removal of spawning tasks, so no need to check whether we can move the listener and param
* removal of control flow

By doing this, much less needs to be checked by the compiler during macro expansion, resulting in a more pleasant developer experience.

I have since lost the plots, but we saw a decrease of ~10 minutes for 1000 functions to roughly ~20 seconds. Because the procedural macro would be in its own crate and not need to be tinkered with and recompiled often, I snapped the chalk line and called this good enough.

## Integration into `up-transport-vsomeip-rust`

I detailed a bit more of the integration in [another post](002-interop-cpp-rust.md), but I'll touch on it a bit here.

### The proc macro crate

The [vsomeip-proc-macro](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/tree/main/vsomeip-proc-macro) is more-or-less similar to the second, more performant example I showed above. There's a bit more machinery around spawning a `tokio` `task` to call the `UListener` which I could go into, but I'll leave this for another day.

### Integration

As I mentioned earlier we now have a pool of these `extern "C" fn`s that we want to ensure are used _at most once_ for the duration they are registered.

I concentrated the mechanisms for this within [`message_handler_registry.rs`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/main/up-transport-vsomeip/src/storage/message_handler_registry.rs). I'll walk through the key portions:

* the [`ProcMacroMessageHandlerAccess`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L51) shim by which the proc macro can access the [`InMemoryMessageHandlerRegistry`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L153)
* the [`InMemoryMessageHandlerRegistry`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/storage/message_handler_registry.rs#L153) by which we can then retrieve new `MessageHandlerFnPtr` to feed into the vsomeip library

I'll pause just short of walking us through the management of the pools of `extern "C" fn`s to keep us focused on the procedural macro usage and revisit that in detail another day.

## And that's it

To recap! Use the least powerful tool for the job as it's likely the easiest to understand.

However -- in our case the [vsomeip](https://github.com/COVESA/vsomeip) needed and we must provide an `extern "C" fn` in order to ping that callback when a message is received. In order to provide good ergonomics around those for _most_ of the codebase of [`up-transport-vsomeip-rust`]() I used Rust's procedural macros to generate a pool of these functions and a management mechanism around them to ensure that we use each function no more than once and return to the pool when finished with it.

All-in-all I found writing Rust proc macros to be a fairly easy and mundane affair. The [`syn`](https://crates.io/crates/syn) and [`quote`](https://crates.io/crates/quote) crates make constructing proc macros out of more-or-less normal Rust code really nice.

Thanks for reading and happy Rust macro usage!

