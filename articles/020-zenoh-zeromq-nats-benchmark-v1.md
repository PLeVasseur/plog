# Benchmarking Zenoh vs ZeroMQ vs NATS - v1

[David Murray](https://www.linkedin.com/in/davidstatheros/) of [Statherós](https://statheros.tech/)reached out to me on LinkedIn to discuss working with his team. What I found striking and interesting is how deep David and team seemed to go and how thoughtful they were.

We chatted for a bit about [Eclipse uProtocol](https://github.com/eclipse-uprotocol), where I'm a maintainer on the protocol-bridging component [up-streamer-rust](https://github.com/eclipse-uprotocol/up-streamer-rust). I explained how the uStreamer service was important to bridge between underylying protocols like [Zenoh](https://zenoh.io/) and [SOME/IP](https://some-ip.com/).

David formed interest in learning how Zenoh stacks up against a middleware they currently use: [NATS](https://nats.io/). He asked me to take an existing benchmark he had for NATS, then expand out to Zenoh and once I had complete that, to also include [ZeroMQ](https://zeromq.org/).

I delivered the benchmark results as shown in this article. In a follow-up article, I'll share further work one of Statherós' engineers Luke Petherbridge diving deeper.

## Benchmark Setup v1

* Conversion from `nats.rs` over to Zenoh + NATS benchmark setup
* Follow-up to add ZeroMQ and my findings

## Benchmark Results v1

### Tables

### Plots

### Statistics

