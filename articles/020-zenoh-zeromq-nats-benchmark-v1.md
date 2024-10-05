# Benchmarking Zenoh vs ZeroMQ vs NATS - v1

[David Murray](https://www.linkedin.com/in/davidstatheros/) of [Statherós](https://statheros.tech/)reached out to me on LinkedIn to discuss working with his team. What I found striking and interesting is how deep David and team seemed to go and how thoughtful they were.

We chatted for a bit about [Eclipse uProtocol](https://github.com/eclipse-uprotocol), where I'm a maintainer on the protocol-bridging component [up-streamer-rust](https://github.com/eclipse-uprotocol/up-streamer-rust). I explained how the uStreamer service was important to bridge between underylying protocols like [Zenoh](https://zenoh.io/) and [SOME/IP](https://some-ip.com/).

David formed interest in learning how Zenoh stacks up against a middleware they currently use: [NATS](https://nats.io/). He asked me to take an existing benchmark he had for NATS, then expand out to Zenoh and once I had complete that, to also include [ZeroMQ](https://zeromq.org/).

I delivered the benchmark results as shown in this article. In a follow-up article, I'll share further work one of Statherós' engineers [Luke Petherbridge](https://www.linkedin.com/in/lucaspetherbridge/) did, diving deeper.

## Benchmark Setup v1

David provided a `nats.rs` benchmark file which used the [criterion](https://crates.io/crates/criterion) benchmarking crate.

I set up a library crate [zenoh-benchmark](https://github.com/PLeVasseur/zenoh-benchmark) with the necessary dependencies like criterion, zenoh, async-nats, and prost (a ProtoBuf crate which would let us serialize the message struct to a `Vec<u8>`).

### Seperating out common setup parameters

I moved the [common portions](https://github.com/PLeVasseur/zenoh-benchmark/blob/main/src/lib.rs) of the benchmarks to the library crate, to ensure that we used a common setup for each benchmark:

```rust
pub const NATS_URL: &str = "nats://127.0.0.1:4222";
pub const NUM_MESSAGES: u64 = 1000;
pub const DURATION: u64 = 30;
pub const INPUT: &str = "Input";

#[derive(prost::Message)]
#[must_use]
pub struct TestMessage {
    #[prost(string, tag = "1")]
    id: String,
    #[prost(string, tag = "2")]
    request_id: String,
    #[prost(string, tag = "3")]
    correlation_id: String,
    #[prost(string, tag = "4")]
    source_id: String,
    #[prost(string, tag = "5")]
    target_id: String,
    #[prost(bytes = "vec", tag = "6")]
    content: Vec<u8>,
}

// Create a test message for publishing/requesting
pub fn test_message(source_id: String) -> TestMessage {
    TestMessage {
        id: "73aea97e-af46-4e54-bae4-c33a10466f99".into(),
        request_id: "bc5272c9-37ed-4ba9-afcc-5c18ed8abe31".into(),
        correlation_id: "a09bdd8c-18d9-4c4a-80ca-707f225e4ce3".into(),
        source_id,
        target_id: "test".into(),
        content: vec![0; 182],
    }
}
```

### Benchmark flow

#### `nats_benchmark.rs`

Let's break down [`nats_benchmark.rs`](https://github.com/PLeVasseur/zenoh-benchmark/blob/main/benches/nats_benchmark.rs) to understand what's happening.

This is the primary functionality that we're going to be testing. We're going to call [`publish()`](https://docs.rs/async-nats/1.37.0/async_nats/client/struct.Client.html#method.publish), passing in the `subject` which we want to send over, `INPUT` and the `payload` which is formed from the `test_message()` function. We will call `publish()` the amount of times passed in as `num_messages`.

```rust
async fn nats_pubsub(client: async_nats::Client, num_messages: u64) {
    for _ in 0..num_messages {
        client
            .publish(
                INPUT.to_string(),
                Message::encode_to_vec(&test_message("nats pubsub".into())).into(),
            )
            .await
            .expect("valid send message");
    }
}
```

Here's benchmark in which we perform setup of the benchmark itself, outlining the input size for the throughput to let us measure the message throughput and the configured measurement time.

We also set up the `async-nats` `Client` with the URL of the NATS server.

We then benchmark the `nats_pubsub` function. We do so by calling [`to_async()`](https://docs.rs/criterion/latest/criterion/struct.Bencher.html#method.to_async), passing in the Tokio `Runtime` to get an [`AsyncBencher`](https://docs.rs/criterion/latest/criterion/struct.AsyncBencher.html), allowing us to call an `async` function within the `.iter()` function.

```rust
fn pubsub_benchmark_no_mutex(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    env_logger::init();

    let mut group = c.benchmark_group("Pub-Sub");
    group.throughput(Throughput::Elements(NUM_MESSAGES));
    group.measurement_time(Duration::from_secs(DURATION));

    let connect_opts = async_nats::ConnectOptions::new()
        .retry_on_initial_connect()
        .no_echo()
        .name("nats_pubsub");
    let client = runtime
        .block_on(connect_opts.connect(NATS_URL))
        .expect("valid nats server");
    group.bench_function("nats", |b| {
        b.to_async(&runtime)
            .iter(|| nats_pubsub(client.clone(), NUM_MESSAGES));
    });

    group.finish();
}
```

At the bottom of the file we have a couple of criterion macros. Let's chat about what these are.

`criterion_main!()` expands to a benchmarking harness, meaning that [as outlined in the docs](https://docs.rs/criterion/latest/criterion/macro.criterion_main.html) we'll need to configure this correctly in our Cargo.toml to ensure we disable the benchmark harnesses automatically generated by rustc:

```toml
[[bench]]
name = "nats_benchmark"
harness = false
```

`criterion_group!()` lets us configure the benchmarks which should be in a group and use the same criterion configuration.

```rust
criterion_group!(benches, pubsub_benchmark_no_mutex);
criterion_main!(benches);
```

#### `zenoh_benchmark.rs`

* Conversion from `nats.rs` over to Zenoh + NATS benchmark setup
* Follow-up to add ZeroMQ and my findings

## Benchmark Results v1

### Tables

### Plots

### Statistics

