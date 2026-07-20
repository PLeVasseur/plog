---
layout: post
title: "Preview: A Native Frame Model for up-rust (Owned and Zero-Copy Transport Families)"
date: 2026-07-20
published: false
---

# A native frame model for uProtocol's Rust library — rendered docs preview

*(Draft — flip `published: true` when the pull requests open.)*

Over the last months I've been building an experimental extension to
[up-rust](https://github.com/eclipse-uprotocol/up-rust): a typed
native frame model with two new transport families — an owned-frame
family and a loan-based zero-copy family — plus wire format
vocabulary, a selected-wire transport adapter, and communication-layer
roles over both. Everything is feature-gated and default-off; existing
`UMessage`/`UTransport` users are unaffected.

Ahead of the pull requests, the full rendered API documentation for
the proposed branch is browsable here:

**[petelevasseur.com/up-rust-preview](/up-rust-preview/)**

Start with the guide chapters (the crate root links them): the
transport families overview, the two-path table (codec path: one
serialization, zero copies; stable path: zero serializations), the
wires chapter, and the trait map that places every role trait.

The work rides on a one-commit companion change to
[up-spec](https://github.com/eclipse-uprotocol/up-spec): three
optional open payload-encoding identity fields on `UAttributes`, so
new payload encodings no longer require editing a closed enum.

Provenance: the preview is rendered from branch `native-frame-model`
(up-rust `b6b99c6d`) with its up-spec companion (`f0e9b17`),
docs.rs-style pinned-nightly, all features, deterministic build. It
carries a banner and `noindex` on every page and will come down once
the PRs resolve. Validation behind it, for the curious: per-commit
dual-toolchain gates across the nine-commit series, a 298-card
feature powerset, Miri, compile-fail typestate proofs, and a
2,160-row cross-transport functional matrix at full pass.
