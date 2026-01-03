---
layout: post
title: "Gracefully Winding Down a C++ Library's Resources From Rust (with an Async Twist)"
date: 2024-08-30
---

# Gracefully Winding Down a C++ Library's Resources From Rust (with an Async Twist)

In [Eclipse uProtocol](https://github.com/eclipse-uprotocol) we built our [uP-L1 Transport](https://github.com/eclipse-uprotocol/up-spec/tree/main/up-l1) library over SOME/IP on top of the [COVESA vsomeip](https://github.com/COVESA/vsomeip) C++ library: [`up-transport-vsomeip-rust`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust).

The goal here being to be able to speak from high compute devices to mechatronics devices (think e.g. brake controllers or IMUs) over SOME/IP, but with a uProtocol API exposed so that the `UPTransportVsomeip` can be plugged into our uStreamer library: [`up-streamer-rust`](https://github.com/eclipse-uprotocol/up-streamer-rust).

From the [`up-transport-vsomeip`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/tree/main/up-transport-vsomeip) crate we call into the [`vsomeip-sys`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/tree/main/vsomeip-sys) crate in order to interact with the C++ vsomeip library. When we do so we spin up vsomeip `application` instances and register message handlers with `application`s to react to in-coming message.

Rather than using the default Rust `Drop` impl, which would be sufficient if we had all our resources been managed within Rust, I needed to implement the `Drop` trait for `UPTransportVsomeip` to properly unregister all message handlers and wind down the vsomeip `application`s.

There's a further twist introduced here related to our use of `tokio`, the most commonly used Rust async framework and runtime. Because we wanted the ability to let users customize the `tokio::runtime::Runtime`, we need to not implement this as a `lazy_static`, but instead carefully control the bringup and shutdown of the `Runtime`. There's a few wrinkles introduced by doing so that we'll get to.

## The vsomeip `application`

The concept here is that we have one or more vsomeip `application`s running, where we in Rust own one thread that has been parked to let the vsomeip library own it and the vsomeip library has started two of its own threads to upon which to put work when we call functions on the `application`. (The number of threads vsomeip should use to service an application is configurable. Two is the default.)

Then when we need to interact with an `application` we use the technique described [in this article](015-rust-async-resource-protection.md). We issue commands over a channel and then in a dedicated thread those commands will be executed upon an `application`.

We use the [`tokio`](https://crates.io/crates/tokio) crate in order to service those commands in an infinite async loop.

## Handling async shutdown correctly

Because of our mix of async and the need to wind down the vsomeip library's resource gracefully in a _non_-async context of the `Drop` impl, I designed our usage of `tokio` in a very particular way.The approach outlined here will avoid the issue `tokio` has with attempting to block the `Runtime`.

In [`get_callback_runtime_handle()`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/lib.rs#L76) we can see the care taken to, from a _non_-async function return us three things:

* a `tokio::runtime::Handle` which we can use to spawn tasks onto
* a `thread::JoinHandle<()>` which we can use to ensure that we've successfully stopped the thread we spin up after we send the command to stop over the ...
* `std::sync::mpsc::Sender<()>` which we send the stop command over so that the `Runtime` will gracefully exit

I'll leave some `PELE` comments below to explain a little further.

```rust
/// Get a dedicated tokio Runtime Handle as well as the necessary infra to communicate back to the
/// thread contained internally when we would like to gracefully shut down the runtime
pub(crate) fn get_callback_runtime_handle(
    runtime_config: Option<RuntimeConfig>,
) -> (
    tokio::runtime::Handle,
    thread::JoinHandle<()>,
    std::sync::mpsc::Sender<()>,
) {
    // PELE: We're setting the number of threads we'll later
    //   ask for tokio to setup on the Runtime for us here
    let num_threads = {
        if let Some(runtime_config) = runtime_config {
            runtime_config.num_threads
        } else {
            DEFAULT_NUM_THREADS
        }
    };

    // PELE: Note that we have chosen to use std::sync::mpsc::channels
    //   since we'll need to be able to use these in a non-async context
    // Create a channel to signal when the runtime should shut down
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel::<()>();
    let (handle_tx, handle_rx) = std::sync::mpsc::channel::<tokio::runtime::Handle>();

    // PELE: Here to avoid the issue of attempting to interact with one Runtime within
    //   another we spawn a thread onto which we will create the Runtime and then
    //   send the handle _back out_
    // Spawn a new thread to run the dedicated runtime
    let thread_handle = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(num_threads as usize)
            .enable_all()
            .build()
            .expect("Unable to create runtime");

        // PELE: Sending the Runtime's handle _back out_ of this thread
        let handle = runtime.handle();
        let handle_clone = handle.clone();
        handle_tx.send(handle_clone).expect("Unable to send handle");

        // PELE: Here we wait until we've been told to shut down the
        //   Runtime and when told to do so, we do it
        match shutdown_rx.recv() {
            Err(_) => panic!("Failed in getting shutdown signal"),
            Ok(_) => {
                // Will force shutdown after duration time if all tasks not finished sooner
                runtime.shutdown_timeout(Duration::from_millis(2000));
            }
        }
    });

    // PELE: grab the runtime_handle off the channel
    let runtime_handle = match handle_rx.recv() {
        Ok(r) => r,
        Err(_) => panic!("the sender dropped"),
    };

    (runtime_handle, thread_handle, shutdown_tx)
}
```

## `Drop` impl

The [`Drop`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/lib.rs#L741) impl for `UPTransportVsomeip` isn't long, but there's some subtleties here that I'll go over.

```rust
impl Drop for UPTransportVsomeip {
    fn drop(&mut self) {
        trace!("Running Drop for UPTransportVsomeip");

        let ue_id = self.storage.get_ue_id();
        trace!("Running Drop for UPTransportVsomeipInnerHandle, ue_id: {ue_id}");

        // PELE: Here we get all of the listener_configs and then
        //   unregister them all (there's some additional detail
        //   in unregister_listener I'm eliding regarding how this is done)
        let storage = self.storage.clone();
        let all_listener_configs = storage.get_all_listener_configs();
        for listener_config in all_listener_configs {
            let (src_filter, sink_filter, comp_listener) = listener_config;
            let listener = comp_listener.into_inner();
            trace!(
                "attempting to unregister: src_filter: {src_filter:?} sink_filter: {sink_filter:?}"
            );
            let unreg_res = self.unregister_listener(&src_filter, sink_filter.as_ref(), listener);
            if let Err(warn) = unreg_res {
                warn!("{warn}");
            }
        }

        trace!("Finished running Drop for UPTransportVsomeipInnerHandle, ue_id: {ue_id}");

        // PELE: Here we signal the shutdown of the Runtime owned by the other thread
        //   by sending an empty message over the std::sync::mpsc::Sender<()> we were
        //   returned by get_callback_runtime_handle()
        trace!("Signalling shutdown of runtime");
        // Signal the dedicated runtime to shut down
        self.shutdown_runtime_tx
            .send(())
            .expect("Unable to send command to shutdown runtime");

        // PELE: Here we wait on the thread's JoinHandle<()> to ensure
        //   that the tokio Runtime has properly been dropped
        // Wait for the dedicated runtime thread to finish
        if let Some(handle) = self.thread_handle.take() {
            handle.join().expect("Thread panicked");
        }

        trace!("Finished Drop for UPTransportVSomeip");
    }
}
```

## `unregister_listener()`

In order to be able to gracefully shut down in the sync context of the `Drop` impl, we need to ensure that any functions we call are _also_ sync. For example, the [`unregister_listener()`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/lib.rs#L267) function is sync, but needs to perform async operations inside of it.

How do we go about that? Well, we use the `tokio::runtime::Handle` we've stored within [`UPTransportVsomeip`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/lib.rs#L129-L143).

```rust
/// UTransport implementation over top of the C++ vsomeip library
///
/// We hold a transport_inner internally which does the nitty-gritty
/// implementation of the transport
///
/// We do so in order to separate the "handle" to the inner transport
/// and the "engine" of the innner transport to allow mocking of them.
pub struct UPTransportVsomeip {
    storage: Arc<UPTransportVsomeipStorage>,
    engine: UPTransportVsomeipEngine,
    point_to_point_listener: RwLock<Option<Arc<dyn UListener>>>,
    config_path: Option<PathBuf>,
    thread_handle: Option<thread::JoinHandle<()>>,
    shutdown_runtime_tx: std::sync::mpsc::Sender<()>,
}
```

The [`unregister_listener()`](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/blob/0d55162da3f7e89ff8456522e1a7c8bd83e051e7/up-transport-vsomeip/src/lib.rs#L267C1-L272C31) function's signature shows it's sync.

```rust
    fn unregister_listener(
        &self,
        source_filter: &UUri,
        sink_filter: Option<&UUri>,
        listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus>
```

So within the function we `block_in_place()` and within that closure then use the `tokio::runtime::Handle` to block on and run the async function `Self::send_to_engine_with_status()`.

```rust
        // Using block_in_place to perform async operation in sync context
        let send_to_engine_res = task::block_in_place(|| {
            self.storage
                .get_runtime_handle()
                .block_on(Self::send_to_engine_with_status(
                    &self.engine.transport_command_sender,
                    TransportCommand::UnregisterListener(
                        src,
                        sink,
                        registration_type.clone(),
                        app_name,
                        tx,
                    ),
                ))
        });
```

## So that's it!

We impl'ed `Drop` for `UPTransportVsomeip` in order to properly unregister the message handler functions from the `application`s and then shut them down. As can be seen, this is complicated a fair bit by the need to also wind down `tokio`'s `Runtime` from within the `Drop` impl.

I can imagine perhaps a cleaner way by, rather than exposing the vsomeip `application` and its APIs directly into Rust, it may be possible to instead write a safe wrapper over top of it so that all registered message handlers are released and the `application` shut down. I'll leave those improvements till another day.

Thanks for reading!

