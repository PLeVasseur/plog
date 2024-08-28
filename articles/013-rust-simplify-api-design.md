# Simplifying Rust API Design for uProtocol uP-L1 Transport trait

In [Eclipse uProtocol](https://github.com/eclipse-uprotocol) there are three layers to the protocol:

* [uP-L1](https://github.com/eclipse-uprotocol/up-spec/tree/main/up-l1): The spec for implementing the `UTransport` API over top of the underlying protocol (e.g. [Zenoh](https://zenoh.io/), [SOME/IP](https://some-ip.com/), ...) so that the underlying protocol can be used by the higher layers
* [uP-L2](https://github.com/eclipse-uprotocol/up-spec/tree/main/up-l2): The layer at which most uEntity (uProtocol app) developers would interact with uProtocol, exposing more developer-focused APIs such as RPC, PubSub, Notifications
  * This level also holds the spec for _dispatchers_, those software whose job it is to move messages around in the system and between devices, such as the [uStreamer](https://github.com/eclipse-uprotocol/up-streamer-rust)
* [uP-L3](https://github.com/eclipse-uprotocol/up-spec/tree/main/up-l3): The layer at which many _core services_ are defined such as uDiscovery, uSubscription, and uTwin. May talk about these in more detail another day.

In the [`up-rust`](https://github.com/eclipse-uprotocol/up-rust) language library at the time, the `UTransport` trait looked like the following:

```rust
/// `UTransport` is the uP-L1 interface that provides a common API for uE developers to send and receive messages.
///
/// Implementations of `UTransport` contain the details for connecting to the underlying transport technology and
/// sending `UMessage` using the configured technology. For more information, please refer to
/// [uProtocol Specification](https://github.com/eclipse-uprotocol/uprotocol-spec/blob/main/up-l1/README.adoc).
#[async_trait]
pub trait UTransport {
    /// Sends a message using this transport's message exchange mechanism.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send. The `type`, `source` and`sink` properties of the [`crate::UAttributes`] contained
    ///   in the message determine the addressing semantics:
    ///   * `source` - The origin of the message being sent. The address must be resolved. The semantics of the address
    ///     depends on the value of the given [attributes' type](crate::UAttributes::type_) property .
    ///     * For a [`PUBLISH`](crate::UMessageType::UMESSAGE_TYPE_PUBLISH) message, this is the topic that the message should be published to,
    ///     * for a [`REQUEST`](crate::UMessageType::UMESSAGE_TYPE_REQUEST) message, this is the *reply-to* address that the sender expects to receive the response at, and
    ///     * for a [`RESPONSE`](crate::UMessageType::UMESSAGE_TYPE_RESPONSE) message, this identifies the method that has been invoked.
    ///   * `sink` - For a `notification`, an RPC `request` or RPC `response` message, the (resolved) address that the message
    ///     should be sent to.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    async fn send(&self, message: UMessage) -> Result<(), UStatus>;
    /// Receives a message from the transport.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic to receive the message from.
    ///
    /// # Errors
    ///
    /// Returns an error if no message could be received. Possible reasons are that the topic does not exist
    /// or that no message is available from the topic.
    async fn receive(&self, topic: UUri) -> Result<UMessage, UStatus>;
    /// Registers a listener to be called for each message that is received on a given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The (resolved) address to register the listener for.
    /// * `listener` - The listener to invoke.
    ///
    /// # Returns
    ///
    /// An identifier that can be used for [unregistering the listener](Self::unregister_listener) again.
    ///
    /// # Errors
    ///
    /// Returns an error if the listener could not be registered.
    async fn register_listener(
        &self,
        topic: UUri,
        listener: Box<dyn Fn(Result<UMessage, UStatus>) + Send + Sync + 'static>,
    ) -> Result<String, UStatus>;

    /// Unregisters a listener for a given topic.
    ///
    /// Messages arriving on this topic will no longer be processed by this listener.
    ///
    /// # Arguments
    ///
    /// * `topic` - Resolved topic uri where the listener was registered originally.
    /// * `listener` - Identifier of the listener that should be unregistered.
    ///
    /// # Errors
    ///
    /// Returns an error if the listener could not be unregistered, for example if the given listener does not exist.
    async fn unregister_listener(&self, topic: UUri, listener: &str) -> Result<(), UStatus>;
}
```

Zooming in, we can see that the method we use for registering and unregistering requires the uEntity developer to "hold on to" an arbitrary `String` returned to us from the `UTransport` so that later we can use said string to call `unregister_listener()`.

[Sagar Shah](https://github.com/sagarsshah), who lead a team working on the [`up-tck`](https://github.com/eclipse-uprotocol/up-tck) (Test Compatability Kit) opened an [issue](https://github.com/eclipse-uprotocol/up-rust/issues/65) on `up-rust` making a note:

> I wanted to discuss a potential overhead to our rust SDK's listener apis. Currently, the register_listener() function within the SDK requires a listener parameter of type Box<dyn Fn(Result<UMessage, UStatus>) + Send + Sync + 'static>, returning a listener string for later use with unregister_listener().
However, I've found it challenging to manage listeners effectively without associating them directly with a UUri and associated function. Other SDKs like Java and Python, which accept the listener function directly for both registration and unregistration, our current Rust SDK requires additional bookkeeping to track listeners.
> 
> Though in real life scenario same uE will rarely listen twice on same UUri but still this can be good correction. Either we expose functions to convert listener function to listener string to uE developer OR we modify rust SDK to match other SDKs.

## How do we make an alternative?

Reading through the issue got me thinking about what alternatives are possible.

I mocked up a few:

1. an [attempt](https://github.com/eclipse-uprotocol/up-rust/pull/67) to use the Rust [`std::any::Any`](https://doc.rust-lang.org/std/any/index.html) type in order to disambiguate between different concrete _types_ of `UListener`
2. a [second attempt](https://github.com/eclipse-uprotocol/up-rust/pull/68) using a wrapper type to avoid the need for a user of `up-rust` to implement a special `as_any()` function
3. a [third attempt](https://github.com/eclipse-uprotocol/up-rust/pull/69) in which we are able to pass in a `std::sync::Arc<dyn UListener>` as-is and leave the work of book-keeping using a `ComparableListener` struct to the implementer of the `UTransport` trait

#1 and #2 both were misguided because they were attempts to use the _type_ of the concrete implementing struct of the `UListener`, but in reality, we needed to be able to use the same _type_ of listener multiple types. This mean that #3 of using the _instance_ of a particular `UListener` trait object wrapped in an `Arc<>` was the way to go. We could then register that same `UListener` trait object to multiple `UUri`s as required by the spec.

## The solution

While working on the second attempt, I saw how much more ergonomic implementation of `UListener` trait objects and `UTransport` traits were by having a single type which could hold onto the `UListener` trait object.

### `UListener` trait

Let's take a look at the details of what I implemented, starting with the [`UListener`](https://github.com/eclipse-uprotocol/up-rust/pull/69/files#diff-2b11318fb2673b25403b992883ee6ac91316e6f11f3c32c1b352caf4e07c79beR89) trait. The goal was to keep this trait which will need to be implemented many times as simple to understand and implement as possible and push all the complexity onto the implementer of the `UTransport` trait, since that centralizes that complexity into one place instead of over N number of uEntities.

You need only implement an `on_receive()` and `on_error()` function in order for the `UTransport` implementation to then know how to pass a `UMessage` or `UListener` respectively for the `UListener` to interact with.

```rust
#[async_trait]
pub trait UListener: 'static + Send + Sync {
    /// Performs some action on receipt of a message
    ///
    /// # Parameters
    ///
    /// * `msg` - The message
    ///
    /// # Note for `UListener` implementers
    ///
    /// `on_receive()` is expected to return almost immediately. If it does not, it could potentially
    /// block further message receipt. For long-running operations consider passing off received
    /// data to a different async function to handle it and returning.
    ///
    /// # Note for `UTransport` implementers
    ///
    /// Because `on_receive()` is async you may choose to either `.await` it in the current context
    /// or spawn it onto a new task and await there to allow current context to immediately continue.
    async fn on_receive(&self, msg: UMessage);

    /// Performs some action on receipt of an error
    ///
    /// # Parameters
    ///
    /// * `err` - The error as `UStatus`
    ///
    /// # Note for `UListener` implementers
    ///
    /// `on_error()` is expected to return almost immediately. If it does not, it could potentially
    /// block further message receipt. For long-running operations consider passing off received
    /// error to a different async function to handle it and returning.
    ///
    /// # Note for `UTransport` implementers
    ///
    /// Because `on_error()` is async you may choose to either `.await` it in the current context
    /// or spawn it onto a new task and await there to allow current context to immediately continue.
    async fn on_error(&self, err: UStatus);
}
```

### The `UTransport` trait

Next, let's look at the [`UTransport`](https://github.com/eclipse-uprotocol/up-rust/pull/69/files#diff-2b11318fb2673b25403b992883ee6ac91316e6f11f3c32c1b352caf4e07c79beR133) implementation. Again, the idea here was to make the API here very simple so that when a user of a given `UTransport` implementation will only have to:

* initialize a struct which implements `UListener`
* wrap that initialized struct in a [`std::sync::Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html)

An `Arc` is a Rust type that lets us put the memory not on the stack, but on the heap instead and get a pointer back for it. Specifically, `Arc` allows us to share that pointer between different threads easily, by just calling `.clone()` to bump the reference count. The memory is in use until there are no further references to the `Arc`, and once that is done then the memory is reclaimed.

```rust
/// [`UTransport`] is the uP-L1 interface that provides a common API for uE developers to send and receive messages.
///
/// Implementations of [`UTransport`] contain the details for connecting to the underlying transport technology and
/// sending [`UMessage`][crate::UMessage] using the configured technology. For more information, please refer to
/// [uProtocol Specification](https://github.com/eclipse-uprotocol/uprotocol-spec/blob/main/up-l1/README.adoc).
#[async_trait]
pub trait UTransport {
    /// Sends a message using this transport's message exchange mechanism.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send. The `type`, `source` and`sink` properties of the [`crate::UAttributes`] contained
    ///   in the message determine the addressing semantics:
    ///   * `source` - The origin of the message being sent. The address must be resolved. The semantics of the address
    ///     depends on the value of the given [attributes' type](crate::UAttributes::type_) property .
    ///     * For a [`PUBLISH`](crate::UMessageType::UMESSAGE_TYPE_PUBLISH) message, this is the topic that the message should be published to,
    ///     * for a [`REQUEST`](crate::UMessageType::UMESSAGE_TYPE_REQUEST) message, this is the *reply-to* address that the sender expects to receive the response at, and
    ///     * for a [`RESPONSE`](crate::UMessageType::UMESSAGE_TYPE_RESPONSE) message, this identifies the method that has been invoked.
    ///   * `sink` - For a `notification`, an RPC `request` or RPC `response` message, the (resolved) address that the message
    ///     should be sent to.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    async fn send(&self, message: UMessage) -> Result<(), UStatus>;
    /// Receives a message from the transport.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic to receive the message from.
    ///
    /// # Errors
    ///
    /// Returns an error if no message could be received. Possible reasons are that the topic does not exist
    /// or that no message is available from the topic.
    async fn receive(&self, topic: UUri) -> Result<UMessage, UStatus>;
    /// Registers a listener to be called for each message that is received on a given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The (resolved) address to register the listener for.
    /// * `listener` - The listener to invoke. Note that we do not take ownership to communicate
    ///                that a caller should keep a copy to be able to call `unregister_listener()`
    ///
    /// # Errors
    ///
    /// Returns an error if the listener could not be registered.
    ///
    async fn register_listener(
        &self,
        topic: UUri,
        listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus>;

    /// Unregisters a listener for a given topic.
    ///
    /// Messages arriving on this topic will no longer be processed by this listener.
    ///
    /// # Arguments
    ///
    /// * `topic` - Resolved topic uri where the listener was registered originally.
    /// * `listener` - Identifier of the listener that should be unregistered. Here we take ownership
    ///                to communicate that this listener is now finished.
    ///
    /// # Errors
    ///
    /// Returns an error if the listener could not be unregistered, for example if the given listener does not exist.
    async fn unregister_listener(
        &self,
        topic: UUri,
        listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus>;
}
```

### Pushing The Complexity into `UTransport` implementations

So then if we're keeping things simpler for the users of the `UTransport` implementation, what do things look like now for the implementers of the `UTransport`s?

#### `ComparableListener`

They key enabler here is the [`ComparableListener`](https://github.com/eclipse-uprotocol/up-rust/pull/69/files#diff-2b11318fb2673b25403b992883ee6ac91316e6f11f3c32c1b352caf4e07c79beR286) struct. Note that `ComparableListener` holds an `Arc<dyn Ulistener>`.

```rust
/// A wrapper type around UListener that can be used by `up-client-foo-rust` UPClient libraries
/// to ease some common development scenarios when we want to compare [`UListener`][crate::UListener]
///
/// # Note
///
/// Not necessary for end-user uEs to use. Primarily intended for `up-client-foo-rust` UPClient libraries
/// when implementing [`UTransport`]
///
/// # Rationale
///
/// The wrapper type is implemented such that it can be used in any location you may wish to
/// hold a type implementing [`UListener`][crate::UListener]
///
/// Implements necessary traits to allow hashing, so that you may hold the wrapper type in
/// collections which require that, such as a `HashMap` or `HashSet`
#[derive(Clone)]
pub struct ComparableListener {
    listener: Arc<dyn UListener>,
}
```

We have an associated function `ComparableListener::new()` which consumes an `Arc<dyn UListener>` and initializes a `ComparableListener`.

```rust
impl ComparableListener {
    pub fn new(listener: Arc<dyn UListener>) -> Self {
        Self { listener }
    }
}
```

We impl the `Deref` trait on `ComparableListener` so that we'll be able to call the `UListener` trait functions directly.

```rust
/// Allows us to call the methods on the held `Arc<dyn UListener>` directly
impl Deref for ComparableListener {
    type Target = dyn UListener;

    fn deref(&self) -> &Self::Target {
        &*self.listener
    }
}
```

We also impl the necessary traits to be able to hash a `ComparableListener` and use within useful Rust collections like a `std::sync::HashSet` and `std::sync::HashMap`. This is important to allow us to efficiently store and look up `UListener`s.

The way we ensure that the hash makes sense here is to ensure we hash the `Arc` pointer which is pointing to the `UListener` . We use pointer equality to make sure that two `ComparableListener`s are equal only if they point to the same instance of `UListener`.

```rust
/// The `Hash` implementation uses the held `Arc` pointer so that the same hash should result only
/// in the case that two [`ComparableListener`] were constructed with the same `Arc<T>` where `T`
/// implements [`UListener`][crate::UListener]
impl Hash for ComparableListener {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.listener).hash(state);
    }
}

/// Uses pointer equality to ensure that two [`ComparableListener`] are equal only if they were
/// constructed with the same `Arc<T>` where `T` implements [`UListener`][crate::UListener]
impl PartialEq for ComparableListener {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.listener, &other.listener)
    }
}

impl Eq for ComparableListener {}
```

#### Example of how to implement a `UTransport`

I wrote some unit tests to make sure that the `UTransport` trait was workable.

For example, here's a [`UPClientFoo`](https://github.com/eclipse-uprotocol/up-rust/pull/69/files#diff-2b11318fb2673b25403b992883ee6ac91316e6f11f3c32c1b352caf4e07c79beR335) which implements `UTransport`.

Now we're able to hold a `Arc<Mutex<HashMap<UUri, HashSet<ComparableListener>>>>` to be able to efficiently look up `ComparableListener`s which are registered for a given `UUri`.

```rust
    struct UPClientFoo {
        #[allow(clippy::type_complexity)]
        listeners: Arc<Mutex<HashMap<UUri, HashSet<ComparableListener>>>>,
    }
```

Next let's look at an example of how to implement `UTransport` on `UPClientFoo`.

Here's how we implement `register_listener()`. Note how we can use `ComparableListener::new(listener)` in order to hold the `UListener` trait object and then store this `ComparableListener` in `self.listeners`.

```rust
    #[async_trait]
    impl UTransport for UPClientFoo {

       // ... snip ...

        async fn register_listener(
            &self,
            topic: UUri,
            listener: Arc<dyn UListener>,
        ) -> Result<(), UStatus> {
            let mut topics_listeners = self.listeners.lock().unwrap();
            let listeners = topics_listeners.entry(topic).or_default();
            let identified_listener = ComparableListener::new(listener);
            let inserted = listeners.insert(identified_listener);

            return match inserted {
                true => Ok(()),
                false => Err(UStatus::fail_with_code(
                    UCode::ALREADY_EXISTS,
                    "UUri + UListener pair already exists!",
                )),
            };
        }

       // ... snip ...

    }
```

Here's how we can implement `unregister_listener()`. Because we are able to hold the `UListener` trait object in `ComparableListener`, we're able to efficiently look up and remove it from `self.listeners`.

```rust
    #[async_trait]
    impl UTransport for UPClientFoo {

       // ... snip ...

        async fn unregister_listener(
            &self,
            topic: UUri,
            listener: Arc<dyn UListener>,
        ) -> Result<(), UStatus> {
            let mut topics_listeners = self.listeners.lock().unwrap();
            let listeners = topics_listeners.entry(topic.clone());
            return match listeners {
                Entry::Vacant(_) => Err(UStatus::fail_with_code(
                    UCode::NOT_FOUND,
                    format!("No listeners registered for topic: {:?}", &topic),
                )),
                Entry::Occupied(mut e) => {
                    let occupied = e.get_mut();
                    let identified_listener = ComparableListener::new(listener);
                    let removed = occupied.remove(&identified_listener);

                    match removed {
                        true => Ok(()),
                        false => Err(UStatus::fail_with_code(
                            UCode::NOT_FOUND,
                            "UUri + UListener not found!",
                        )),
                    }
                }
            };
        }
    }
```

## And it works in practice

Along with this change to `up-rust`, I also implemented this change in [`up-transport-zenoh-rust`](https://github.com/eclipse-uprotocol/up-transport-zenoh-rust) in a [draft PR](https://github.com/eclipse-uprotocol/up-transport-zenoh-rust/pull/19). This was important to ensure that `UListener`, `UTransport`, and `ComparableListener` work together in practice to be able to implement a `UTransport`.

Eventually my PR drifted out of date a little bit and it made more sense for [CY](https://github.com/evshary), the maintaner of `up-transport-zenoh-rust` to pick it up, so he implemented the change in [this PR](https://github.com/eclipse-uprotocol/up-transport-zenoh-rust/pull/24).

Sagar was able to now have a more unified API when writing and integrating a `UTransport` trait implementation [over sockets](https://github.com/eclipse-uprotocol/up-transport-socket/tree/main/rust) they need when exercising the `up-rust` crate.

## Collaboration on open source and cross-pollination

One thing I really like about the [`up-cpp`](https://github.com/eclipse-uprotocol/up-cpp) design that [Greg Medding](https://github.com/gregmedd) came up with is their concept of a [`Connection`]() which allows the entity registering a callback with their uP-L1 Transport implementation to drop their end of the `Connection` in order to _disallow_ the calling of the callback and an implicit unregistration.

Writing this article has been great, because it gave me a chance to reflect back on the design choice I made and consider if it would be better to instead pass in a [`std::sync::Weak`](https://doc.rust-lang.org/stable/std/sync/struct.Weak.html) instead of a `std::sync::Arc`. If we passed in a `Weak<dyn UTransport>` to `UTransport::register_listener()`, then anytime we'd attempt to call that `Uistener::on_receive()` we'd have to first call [`Weak::upgrade()`](https://doc.rust-lang.org/stable/std/sync/struct.Weak.html#impl-Weak%3CT,+A%3E-2) in order to safely check if the reference count is non-zero before converting to an `Arc<dyn UTransport>`, which would give all the same benefits.

I opened up [an issue](https://github.com/eclipse-uprotocol/up-rust/issues/200) on `up-rust` for further discussion with the other contributors, feel free to check it out.

And that's a wrap! Thanks for reading.

