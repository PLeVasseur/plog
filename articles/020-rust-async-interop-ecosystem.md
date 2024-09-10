# Rust Async Ecosystem Interoperability & Glue-ability

There are a lot of articles and nice tutorial videos written about how the [`Future`](https://doc.rust-lang.org/std/future/trait.Future.html) trait being in `std`, while allowing the actual executors and async frameworks to grow outside of the standard library leads to innovation and flexibility.

I'm not experienced enough with the details, so I thought I'd write an article closer to the perspective of a somewhat new user of the Rust async ecosystem based on my experiences so far implementing software for our Software Defined Vehicle framework [Eclipse uProtocol](https://github.com/eclipse-uprotocol).

I'll go through two parts:

1. Async ecosystem interoperability
2. Async ecosystem glue-ability

## Async ecosystem interoperability

TODO: Talk about how when writing integration tests for up-streamer confirmed they can be run using an executor with executor-agnostic channel mechanism

When writing integration tests for [`up-streamer-rust`](https://github.com/eclipse-uprotocol/up-streamer-rust/)'s [`up-streamer`](https://github.com/eclipse-uprotocol/up-streamer-rust/tree/main/up-streamer) crate that's a generic library, not tied to any underlying transport protocol (such as Zenoh or SOME/IP), I had a need to write a mock transport that could be used to exercise the logic in integration tests.

I chose [`async-std`](https://crates.io/crates/async-std) for the internals of `up-streamer`, as the [Zenoh](https://zenoh.io/) middleware was using it as its async framework. However, I learned of [plans in its 1.0.0 release](https://www.zettascale.tech/news/zenoh-and-whats-next-in-2024/) to switch Zenoh to use `tokio`.

So I wanted to write the integration test mock in an async framework agnostic way.

### Requirements

I needed the ability to essentially have a way to "broadcast" data out so that multiple recipients could receive it in order to mock the [uP-L1 Transport]() spec which is reflected in `up-rust`'s [`UTransport`]() trait. By doing so I could then plug this mocked transport into `up-streamer`'s [`Endpoint`]() and then be able to test the various flows.

## Async ecosystem glue-ability

TODO: integration of the uStreamer into a Zenoh plugin we were able to easily slip in another tokio threadpool

