# Rust Async Pattern For Resource Protection using Channels and Enums

A pattern that's quite useful in Rust when doing asynchronous programming with a resource which should only be accessed by one thread is to use Rust channels and enums for communication with that dedicated thread.

There are two instances where I have used this pattern in [Eclipse uProtocol](https://github.com/eclipse-uprotocol):

1. The initial implementation of the `up-streamer` crate, detailed in this [post](010-writing-async-rust.md).
2. The current implementation of the `up-transport-vsomeip-rust` crate.

Let's discuss the general principle and then I'll show the specifics of the `up-transport-vsomeip-rust` crate in a follow-up post.

## General principle

Let's say that we have some resource which we either _must_ or _want_ to have accessible. It could be that it's a struct or trait object which is not thread safe, for example.

In that case, we can leverage Rust's robust ecosystem around channels and Rust's enum capabilities to create rather robust control mechanisms around that resource.

Imagine we are commanding a grasping robot arm and wish to have only a single thread have control over the actuation for safety purposes.

### The command enum(s)

First we can define one or more enums, nesting them to describe what the commands are and what information they must command. 

We might come up with enums which look like the following.

```rust
enum Finger {
  Thumb,
  Pointer,
  Middle,
  Index,
  Pinky
}

enum HandCommand {
  FlexWrist(f64), // number of radians to flex wrist
  TapFinger(Finger),
  // ... more ...
}

enum ArmCommand {
  RotateBase(f32), // radians to swivel base of arm
  // ... more ...
}

enum ActuationCommand {
  Hand(HandCommand),
  Arm(ArmCommand)
}
```

### The actuation struct

Let's imagine we have an `Actuator` which can perform the commands.

```rust
struct Actuator {
  // ... snip ...
}

impl Actuator {
  fn flex_wrist(flex: f64) { /* snip */ }

  fn tap_finger(to_tap: Finger) { /* snip */ }

  fn rotate_base(swivel: f32) { /* snip */ }
}
```

### The command loop

The previous post gave an example using `async-std`. The real-world example I'll give later will use `tokio`, so let's use `tokio` for this general principle section. Here we're going to spawn a thread with a dedicated `tokio` `Runtime` to process commands received.

I'll add some comments starting with `PELE` to explain what's going on.

```rust
// PELE: We take a Receiver which is the receiving end of a Rust channel
//   There are many channel libraries which could serve this purpose, e.g.
//   tokio::sync::mpsc::Receiver
fn actuation_command_loop(rx_cmd: Receiver<ActuationCommand>) {
  // PELE: Spawn a dedicated thread onto which to run the command loop
  thread.spawn(move || {
    // PELE: Make a tokio Runtime which is dedicated to the spawned thread
    let runtime = Builder::new_current_thread()
      .enable_all()
      .build()
      .expect("Failed to create tokio runtime");

    // PELE: Perform a blocking operation on this thread-dedicated Runtime
    runtime.block_on(async move {
      // PELE: ... and create our thread-local Actuator
      let actuator = Actuator::new(/* ... */);

      // PELE: ... to then go into the infinite loop processing commands
      //   using the Actuator
      while let Some(cmd) = rx_cmd.recv().await {
        // PELE: We handle the commands by then dispatching
        //   into the Actuator functions
        match cmd {
          ActuationCommand::Hand(hand_cmd) => {
            match hand_cmd {
              HandCommand::FlexWrist(flex) => {
                actuator.flex_wrist(flex);
              }
              HandCommand::TapFinger(finger) => {
                actuator.tap_finger(finger);
              }
            }
          }
          ActuationCommand::Arm(arm_cmd) => {
            match arm_cmd {
              ArmCommand::RotateBase(swivel) => {
                actuator.rotate_base(swivel);
              } 
            }
          }
        }
      }
    });
  });
}
```

### Initializing the loop

We need to create a channel `Sender` and `Receiver` through which to communicate commands before starting the command loop. Feel free to use whichever channel library you would like, there are many choices. Here I'm using `tokio::sync::mpsc::channel`.

```rust
let (rx, tx) = channel(10000);

// PELE: Transfer ownership of the Receiver into the command loop
actuation_command_loop(rx);
```

### Sending a command

We can now send a command using the `tx` `Sender`.

```rust
tx.send(ActuationCommand::Hand(HandCommand::TapFinger(Finger::Pointer))).await;
```

The command loop will process the commands in the order they are sent. In fact, we can clone the `Sender` and pass it off to multiple different threads, since I imagine that we'll have different parts of the system which need to send `ActuationCommands`.

## General principle with returning status

Now this will work so long as we only need to fire off commands to the command loop. However, _most_ of the time we'll want to receive back something such as a status or some retrieved value.

### Returning contents _from_ the command loop

Let's flesh the example out a bit further by considering that by handing over a one-shot use channel within the enum members they can then use to communicate back. Here again, there are different options to choose from. I'll use a [`tokio::sync::oneshot::channel`](https://docs.rs/tokio/latest/tokio/sync/oneshot/fn.channel.html).

```rust
struct Status {
  InvalidCommandState,
  // ...
}

type Pressure = f64;

enum Finger {
  Thumb,
  Pointer,
  Middle,
  Index,
  Pinky
}

enum HandCommand {
  FlexWrist(f64, oneshot::Receiver<Result<(), Status>>), // number of radians to flex wrist
  TapFinger(Finger, oneshot::Receiver<Result<Pressure, Status>>),
  // ... more ...
}

enum ArmCommand {
  RotateBase(f32, oneshot::Receiver<Result<(), Status>>), // radians to swivel base of arm
  // ... more ...
}

enum ActuationCommand {
  Hand(HandCommand),
  Arm(ArmCommand)
}
```


### Sending a command

The sending of the command itself doesn't change much. What does change is the semantics we want to have around waiting on receiving the result back from the command loop. What to do here can depend on the specifics of your application. One option is to await for a certain time and then fail if timed out.

```rust
const MAX_WAIT_DUR_MS: u64 = 30;

let (res_rx, res_tx) = oneshot::channel();

tx.send(ActuationCommand::Hand(HandCommand::TapFinger(Finger::Pointer, res_tx))).await;

match timeout(Duration::from_millis(MAX_WAIT_DUR_MS), res_rx) {
  Ok(Ok(pressure)) => {/* do something with returned pressure */}
  Ok(Err(e)) => {/* do something like log error we failed to receive status back */}
  Err(e) => {/* here we know the deadline exceeded */}
}
```

### Gracefully closing down the command loop

You will likely want another command which, when received, will notify the command loop to gracefully close down. Doing so is left as an exercise to the reader!

## And that's it!

I've used this pattern two times when trying to protect a resource which was not thread-safe and it worked well in both cases. Sometimes when interacting with those resources it's possible to make the resource thread-safe. Sometimes it's not, like when binding to and using another library through FFI. Nice to have another tool in the tool belt.

