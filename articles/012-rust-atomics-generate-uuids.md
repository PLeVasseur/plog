---
layout: post
title: "Using Rust Atomics for Consistency under High Contention"
date: 2024-08-23
---

# Generating Unique uProtocol UUIDs under Thread Contention using Rust Atomics and the Singleton Pattern

In a previous release of the [Eclipse uProtocol](https://github.com/eclipse-uprotocol), [1.5.7](https://github.com/eclipse-uprotocol/up-spec/tree/v1.5.7), we used a [UUIDv8](https://en.wikipedia.org/wiki/Universally_unique_identifier#:~:text=A%20Universally%20Unique%20Identifier%20(UUID,Acronym)) which looked like this:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         unix_ts_ms                            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|           unix_ts_ms          |  ver  |         counter       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|var|                          rand_b                           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                           rand_b                              |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

I'll break this down a bit and here's a [link](https://github.com/eclipse-uprotocol/up-spec/blob/v1.5.7/basics/uuid.adoc) to that previous spec, which goes into more detail.

## uProtocol v1.5.7 UUID Spec

* `unix_ts_ms`: the number of milliseconds elapsed time since the [Unix epoch](https://www.epoch101.com/#:~:text=The%20Unix%20epoch%20is%20the,POSIX%20time%2C%20or%20Unix%20timestamp.)
* `counter`: if that millisecond from `unix_ts_ms` would already have been used, we make sure to also increment this
* `ver`: the version of UUID, so in this case it's `8` since we're using UUIDv8
* `var`: have to set to `10b` according to Section 4.1 of [draft-ietf-uuidrev-rfc4122bis](https://datatracker.ietf.org/doc/draft-ietf-uuidrev-rfc4122bis/)
* `rand_b`: a 62 bit pseudo-random number generated per uE. **MUST NOT** change during the lifecycle of the uEntity (i.e. the app communicating over uProtocol)

## The issue

At the time, the [method](https://github.com/eclipse-uprotocol/up-rust/blob/a30d3655ab13f8d97815280d718f4891f693ed2d/src/uuid/uuidbuilder.rs) of generating `UUID`s had a bit of a drawback. You see, we allowed the developer using [`up-rust`](https://github.com/eclipse-uprotocol/up-rust) to construct as many `UUIDBuilder`s as they wanted from which to generate `UUID`s.

The problem is that this would allow a developer writing a Rust uEntity to accidentally violate the UUID spec, namely this section I wrote above:

> * `rand_b`: a 62 bit pseudo-random number generated per uE. **MUST NOT** change during the lifecycle of the uEntity (i.e. the app communicating over uProtocol)

[Steven Hartley](https://github.com/stevenhartley), Eclipse uProtocol project lead, found this and I raised [an issue](https://github.com/eclipse-uprotocol/up-rust/issues/58) to discuss with the other contributors. After some discussion among other contributors and maintainers, we agreed that using a singleton pattern made sense to ensure that a developer using the `UUIDBuilder` would not accidentally violate spec.

## The resolution

I wrote up and got merged a [PR](https://github.com/eclipse-uprotocol/up-rust/pull/74) which resolved the issue. I'll dive a little deeper here into how this was done.

### The singleton

I implemented the singleton pattern in a way that would not get in the way of a developer using the `UUIDBuilder` by holding the internal state within the [`UUIDBuilder::build()`](https://github.com/eclipse-uprotocol/up-rust/pull/74/files#diff-d0fb7a31d66f5ab9c710692b413e1db44b33f47cd10c843767c3f474f731795aR42) function.

We use a `once_cell::sync::Lazy` which will let us ensure that `UUIDBuilder::new()` is called only once to initialize with a consistent `rand_b` portion and then call `build_internal()` on the `UUIDBUILDER_SINGLETON`.

```rust
use once_cell::sync::Lazy;
use rand::random;

impl UUIDBuilder {
    /// Builds a new UUID with consistent `rand_b` portion no matter which thread or task this is
    /// called from. The `rand_b` portion is what uniquely identifies this uE.
    ///
    /// # Returns
    ///
    /// UUID with consistent `rand_b` portion, which uniquely identifies this uE
    pub fn build() -> UUID {
        static UUIDBUILDER_SINGLETON: Lazy<UUIDBuilder> = Lazy::new(UUIDBuilder::new);
        UUIDBUILDER_SINGLETON.build_internal()
    }

    // ... continued later ...

}
```

Let's now look at [`UUIDBuilder::new()`](https://github.com/eclipse-uprotocol/up-rust/pull/74/files?diff=split&w=0#diff-d0fb7a31d66f5ab9c710692b413e1db44b33f47cd10c843767c3f474f731795aR55), where we generate our `rand_b` portion and set the `ver` and `var` constants. We ensure to mark this as [`pub(crate)`](https://doc.rust-lang.org/reference/visibility-and-privacy.html) visibility so that a developer using `up-rust` cannot access this associated function, but we are able to do so within the `up-rust` crate such as in the `build()` associated function above and for testing purposes.

```rust
    /// Creates a new builder for creating uProtocol UUIDs.
    ///
    /// The same builder instance can be used to create one or more UUIDs
    /// by means of invoking [`UUIDBuilder::build_internal()`].
    ///
    /// # Note
    ///
    /// For internal testing purposes only. For end-users, please use [`UUIDBuilder::build()`]
    pub(crate) fn new() -> Self {
        UUIDBuilder {
            msb: AtomicU64::new(0),
            lsb: random::<u64>() & BITMASK_CLEAR_VARIANT | crate::uuid::VARIANT_RFC4122,
        }
    }
```

With this formulation it's now guaranteed that an end-user will not violate spec and ensure they _always_ get a `rand_b` portion which is consistent throughout the lifecycle of the uEntity.

However, we now need to implement the `build_internal()` function in a way that is performant, even under thread contention when being accessed from multiple threads.

### The Rust Atomics

Next let's dive into how [`UUIDBuilder::build_internal()`](https://github.com/eclipse-uprotocol/up-rust/pull/74/files?diff=split&w=0#diff-d0fb7a31d66f5ab9c710692b413e1db44b33f47cd10c843767c3f474f731795aR80) was implemented.

We need to implement this in a way that is performant even if the developer has multiple threads up or is using async Rust so that we reduce the amount of overhead spent ensuring that we don't have collisions and generate the same `UUID`, given that it's _very possible_ to call `UUIDBuilder::build()` from multiple contexts at the exact same millisecond and end up with the same `counter` value.

#### Benchmarking the options

I considered a couple of options to ensure no collisions:

* a [`std::sync::Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
* a [`std::sync::atomic::AtomicU64`]() to hold the `rand_b` portion and perform a [Compare-and-Swap (CAS)](https://en.wikipedia.org/wiki/Compare-and-swap#:~:text=In%20computer%20science%2C%20compare%2Dand,to%20a%20new%20given%20value.)

In some benchmarking, I found that using Rust atomics led to a significantly lower overhead.

Pulling from my [PR](https://github.com/eclipse-uprotocol/up-rust/pull/74):

> Using the Atomic CAS technique is approximately twice as fast as using a Mutex
> 
> With 10 async tasks, each generating 400 UUIDs all released and run at once
> 
> * Mutex: ~2ms
> * Atomic CAS: ~0.8ms

So the answer seemed pretty settled to use a Compare-and-Swap with an `AtomicU64`. Let's dig into how this worked next.

#### The details

Below is the [`UUIDBuilder::build_internal()`](https://github.com/eclipse-uprotocol/up-rust/pull/74/files?diff=split&w=0#diff-d0fb7a31d66f5ab9c710692b413e1db44b33f47cd10c843767c3f474f731795aR80) function in its entirety. I found it so cool that it could be implemented so succinctly.

I'll be adding some comments prepended with `PELE: #NUMBER` to go over the details of how this works.

```rust
impl UUIDBuilder {

    // ... already shown details...

    /// Creates a new UUID based on a particular instance of [`UUIDBuilder`]
    pub(crate) fn build_internal(&self) -> UUID {
        // We utilize a Compare-and-Swap (CAS) technique here to ensure that we always generate
        // a new UUID safely in any concurrent context
        loop {
            // PELE: 1. We load the most-significant bits, the AtomicU64 holding the
            //   unix_ts_ms and the counter into a local variable using atomic operation ordering
            //   ordering of `Ordering::SeqCst` to ensure that we synchronize this operation across
            //   all threads calling `build_internal()`. Key step!
            let current_msb = self.msb.load(Ordering::SeqCst);

            // PELE: 2. We ensure we grab a fresh time at the top of every loop
            //   We have to bail on the current loop iteration and go to the next one if we have
            //   exhausted all bits of the counter and we need to check each time if we are
            //   in a new millisecond
            let cas_top_of_loop_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("current system time is set to a point in time before UNIX Epoch");

            // PELE: 3. Here we need to convert the u128 returned by duration_since() into
            //   a u64 in preparation for truncating it
            let cas_top_of_loop_millis = u64::try_from(cas_top_of_loop_time.as_millis())
                .expect("current system time is set to a point in time too far in the future");

            // PELE: 4. We then truncate the last 16 bits and slide into position so that we
            //   can compare against the current 48-bit long unix_ts_ms
            let current_timestamp = current_msb >> 16;
            let new_msb;

            // PELE: 5. On the first call to `UUIDBuilder::build_internal()` within a new
            //   millisecond, this check will fail and we will proceed to the else block
            if cas_top_of_loop_millis == current_timestamp {
                // PELE: 5a. Here we do a check on whether we've saturated the counter and
                //   if so, we just continue the loop back to the top hoping for a new millisecond
                //   If all goes well, we pack new_msb

                // If the timestamp hasn't changed, attempt to increment the counter.
                let current_counter = current_msb & MAX_COUNT;
                if current_counter < MAX_COUNT {
                    // Prepare new msb with incremented counter.
                    new_msb = current_msb + 1;
                } else {
                    // this should never happen in practice because we
                    // do not expect any uEntity to emit more than
                    // 4095 messages/ms
                    // so we simply keep the current counter at MAX_COUNT
                    // and wait for the next millisecond to arrive
                    continue;
                }
            } else {
                // PELE: 5b. Here we know we're in a new millisecond so we reset the counter and use
                //   the bit-shifted new cas_top_of_loop_millis to fit into the right 48-bit slot
                //   of new_msb

                // New timestamp, reset counter.
                new_msb = (cas_top_of_loop_millis << 16) & BITMASK_CLEAR_VERSION
                    | crate::uuid::VERSION_CUSTOM;
            }

            // PELE: 6. Here's where the magic happens. We perform the Compare-and-Swap or as
            //   it's called in Rust atomics, the compare_exchange where we will only overwrite
            //   and return the UUID if 
            //   * compare_exchange will only allow us to exchange values if the current value
            //     of the AtomicU64 is the same as current_msb, the value we loaded at the top
            //     of the loop. This prevents us from accidentally overwriting a value we didn't
            //     "own"
            //   * if indeed the current value of the AtomicU64 and current_msb agree, then 
            //     compare_exchange will swap in new_msb with the updated timestamp and/or counter
            //   * here again we use the atomic ordering of Ordering::SeqCst to ensure
            //     synchronization between diffferent threads and with the load operation performed
            //     at the top of the loop

            // Only return if CAS succeeds
            if self
                .msb
                .compare_exchange(current_msb, new_msb, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return UUID::from_u64_pair(new_msb, self.lsb)
                    .expect("should have been able to create UUID for valid timestamp");
            }
        }
    }
}
```

## Testing the new singleton works

As a part of implementing the `UUIDBuilder` as a singleton I implemented [a unit test](https://github.com/eclipse-uprotocol/up-rust/pull/74/files?diff=split&w=0#diff-d0fb7a31d66f5ab9c710692b413e1db44b33f47cd10c843767c3f474f731795aR135) to ensure that the invariants of the UUID uProtocol spec were upheld.

In said unit test, we spin off 10 async tasks where each task does the same thing: loop and call `UUIDBuilder::build()` 400 times.

We then [wait](https://github.com/eclipse-uprotocol/up-rust/pull/74/files?diff=split&w=0#diff-d0fb7a31d66f5ab9c710692b413e1db44b33f47cd10c843767c3f474f731795aR164) for those tasks to finish. I must say here that working with async in Rust may have some rough edges here and there, but is _much_ easier to reason about and work with than in C++.

```rust
        // Await all tasks and collect their results
        let results = futures::future::join_all(tasks).await;
```

The interesting thing may be in the invariants we check are upheld.

### Ensuring no duplicate `UUID`s generated

I've left some comments below to explain a little more detail

```rust
        #[allow(clippy::mutable_key_type)]
        let mut all_uuids = HashSet::new();
        let mut duplicates = Vec::new();
        for local_uuids in results {
            for (task_id, uuid) in local_uuids {
                // PELE: Here we check if insertion into the HashSet succeeds
                //   and if it fails (due to being a duplicate) we insert into
                //   the duplicates Vec
                if !all_uuids.insert(uuid.clone()) {
                    duplicates.push((task_id, uuid));
                }
            }
        }

        // PELE: Here we ensure that the duplicates Vec is empty and if otherwise
        //   we fail the test

        // triggers if we have overrun the counter of 4095 messages / ms
        assert!(
            duplicates.is_empty(),
            "Found {} duplicates. First duplicate from task {}: {:?}",
            duplicates.len(),
            duplicates
                .first()
                .map(|(task_id, _)| *task_id as isize)
                .unwrap_or(-1),
            duplicates.first().map(|(_, uuid)| uuid)
        );
```

### Ensuring we generate _exactly_ the number of `UUID`s expected

Here we ensure that the record we have of `all_uuids` matches the expected length of `num_tasks * uuids_per_task`.

```rust
        // another check which would trigger if we've overrun the counter of 4095 messages / ms
        // since if these are not equal, it means we had duplicate UUIDs
        assert_eq!(
            all_uuids.len(),
            num_tasks * uuids_per_task,
            "Mismatch in the total number of expected UUIDs."
        );
```

## And that's it!

With this approach we were able to ensure that developers of uEntities in Rust could guarantee they follow spec, by _preventing the issue entirely_ via construction.

## The next uProtocol UUID spec

There was further discussion later initiated by [Greg Medding](https://github.com/gregmedd) in an [issue](https://github.com/eclipse-uprotocol/up-spec/issues/170) over moving to a UUIDv7 which will not have `unix_ts_ms`, `counter`, nor a fixed `rand_b` per uEntity, but simply generate a pseudo-random number per UUID we generate. In the end we did choose UUIDv7 for a number of good reasons outlined by Greg in the issue.

I agreed with him on two key points:

* We already have the [source UUri](https://github.com/eclipse-uprotocol/up-spec/blob/da5ca97d3a7541d2fcd52ed010bc3bcca92e46cb/up-core-api/uprotocol/v1/uattributes.proto#L37) which would identify the sender, so there's no real need for the `rand_b` portion to uniquely identify a uEntity sender.
* uProtocol did not have message sequencing guarantees in the spec. Adding message sequencing guarantees _does_ come with trade-offs in other parts of the design. Those sequence guarantees we can leave to the users of uProtocol to ensure in their application, if it's of importance to them.

