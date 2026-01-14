---
layout: post
title: "Curiosity: How I Moved from Senior to Staff Engineer"
date: 2026-01-14
---

# Curiosity: How I Moved from Senior to Staff Engineer

Reflecting on my career a bit and how I got here, the main reason I was able to move from what you might call a "Senior" engineer to a "Staff" engineer is due to curiosity and channeling this effectively.

Curiosity? Isn't that the thing which leads us engineers to rabbit-holing on a problem endlessly? Right. So, onto channeling it effectively.

## Missing deadlines and recovery

Rewind the clock to three years ago when I was involved in an eight week long "stop-the-world" reimagining of an entire Automated Driving product. Development was more-or-less paused on product development as we worked out a shift from primarily map-based to a fusion of map and perception for feeding the planning and control stacks.

I spent around a month scaffolding out the new component we'd need to accomplish the map and perception fusion, with its various modules. I started to talk with our tech lead and manager on the initial pieces needed to get the planning and control stacks fed. Broke the work down.

I then spent the next few months deep-diving on what we'd need to do in order to have a comprehensive solution as a part of our iteration. Checked in from time to time with the tech lead. Started to get nervous. Could see that I was likely not going to hit what I committed to for the iteration. Signaled this to the manager and tech lead.

We had a meeting in which the manager and tech lead communicated they were totally fine with an incomplete solution. The key thing was to start getting the planning and control stacks fed with data so that we could exercise the new product-level solution.

I'll admit that I was initially defensive and off-put that it appeared they hadn't communicated this to me. It took me a couple of weeks to process through what I was feeling. Talking with a friend that had done a good deal of project and people management also helped.[^product-level-thinking]

[^product-level-thinking]: With the benefit of a few more years and a few more projects, I can now see how important this type of product-level thinking is. They needed to derisk the entire product and our particular contribution to it as soon as possible, in case another pivot was needed.

I became interested in how we could recover the situation. I started to break this down in some slides [^runs-on-slides] on what a less comprehensive solution looked like, how much effort it would take, and who I could have work with me to complete the work in the next iteration. [^curious-about-people]

[^runs-on-slides]: It was often said that the company did not produce X, where X was our product, but instead slides and spreadsheets.

[^curious-about-people]: Because I was also curious in what people were up to and their interests, I knew of a team member that would be a shoe-in for this type of fast-moving work without a lot of clear direction.

I scheduled a meeting with the manager and tech lead. The manager and tech lead appreciated the recovery plan, it was greenlit and we began work.

Was everything done the way I would have? No. But -- we hit our target on the next iteration and there was now someone else on the team that left a shared sense of responsibility and ownership for this module.

It took us another six months or so to have vehicles working both in controlled highway and surface street environments, but this goes to show how they were ultimately _incredibly correct_ in their product-level thinking to get something shipped even if not complete. It took around three months for the planning and control teams to integrate our work and then another three months of thorough end to end testing in simulation and on-vehicle.

We were able to continue iterating, with me then pulling in more folks to work on specific modules of the scene fusion aspect and orchestrating their work.

### So what?

Staying curious on how we could accomplish what we set out to do gained me skills in delegation and orchestration that'd come in useful around a year later when this product was cancelled. I had also gained organizational trust and leverage.

A year later when the product was cancelled and our team was searching for a way to use our software we'd built over the last six five years my manager tapped me for running this. I was able to coordinate with upstream and downstream teams, build out a plan and then execute.

The whole "build a recovery plan" skill also came in handy when the team upstream of us fumbled by including no technical staff in discussions we had for three months (despite my downright pleading at times) and then I needed to function as their mini-PM to still ship work on time.

In addition, when our product was ultimately cancelled a year later:
* the tech lead helped me find another spot within the same company to land which involved using the Rust programming language, working on open source[^eclipse-uprotocol]
* the manager helped lobby for me to transition from Senior to Staff when moving roles to another team and I succeeded in doing so

[^eclipse-uprotocol]: This was in a team working on [Eclipse uProtocol](https://uprotocol.org/), in an open-source capacity. One of the greatest nine months of my career! (till this entire org was liquidated ;-;) This work later opened the door to doing more open source and community work for Rust, which was excellent.

## Eclipse SDV Rust Special Interest Group

Was I already busy enough now with working on Eclipse uProtocol and orchestrating the stand-up of a team to do this work within the company in 2024? Yes! But did I see a Rust SIG being created and wanted to see what was going on? Also -- yes!

I saw that Florian Gilcher would be the lead of this Rust SIG, knew that Ferrous Systems had done work to bring Rust to the safety-critical world through Ferrocene and couldn't help myself. [^ferrous-ferrocene]

[^ferrous-ferrocene]: I attended the launch party the prior year in 2022. They know how to do it. The launch party included a live DJ with electronic music. Check out [Ferrocene](https://ferrocene.dev/) if you haven't before. It's developed entirely in open source!

I stayed involved working with Florian and others to shape the scope of the SIG and get it stood up. [^rust-sig]

[^rust-sig]: If I'm honest with you, we're still figuring things out, but I think in 2026 we're moving in the right direction with having "watercooler chat"-style low-stakes talks from folks. It's vectoring the right way and I'm building up a list of folks to come out. Stay tuned.

I was honestly just incredibly interested in how I could help in shaping the future of Rust in Automotive!

### So what?

Welp, after I was laid off from my company, I pinged Florian and let him know I'd be at RustConf 2024 in Montreal, seeing if he'd like to get together for dinner. He took me up on this and we had a lovely (if slightly chilly) dinner outdoors.

At the end of dinner Florian mentioned that the Rust Foundation and industry stakeholders were putting together a thing called the Safety-Critical Rust Consortium. He invited me to come along the next day and told me where to show up.

I showed up and among handing out my "business card" (one of those snake puzzle toys with my contact info on it) and chatting about gaps when it comes to Rust in Safety-Critical I raised my hand when it came to Rust Coding Guidelines.

Over the next couple of months on the Rust Zulip I would ping Joel Marcey and others about getting things stood up. He eventually just said "hey Pete you wanna be the chair of the Coding Guidelines Subcommittee?" To which I said: "sure, if you'll have me!".

It's my opinion that through my efforts in the Safety-Critical Rust Consortium and its Coding Guidelines Subcommittee, as well as my efforts on Eclipse uProtocol and the Rust SIG I made enough of an impression on my current employer to be able to land there.

Staying curious, saying "yes", and being willing to put in the work had opened more doors. This time, thankfully, to employment!

## I lied

This is not only about how to move from "Senior" to "Staff". It's also about why staying curious will help you to take a more active role in your career.

Hopefully the thrust is clear. Staying curious, getting involved, and being the person willing to put in work helped me get where I am.

But -- also knowing when to reconsider, break things down and then help orchestrate work has also been a key skill I've picked up. These skills matter to me more now than ever as I've moved into the role of Lead of the Safety-Critical Rust Consortium last year.[^safety-critical-rust-consortium]

[^safety-critical-rust-consortium]: We're working to make Rust be a first-class citizen when folks want to obtain stronger guarantees around memory-safety, type-safety, and thread-safety. Check out the site [here](https://arewesafetycriticalyet.org/), or our repo [here](https://github.com/rustfoundation/safety-critical-rust-consortium). Our in-progress coding guidelines are deployed [here](https://coding-guidelines.arewesafetycriticalyet.org/), with the repo [here](https://github.com/rustfoundation/safety-critical-rust-coding-guidelines). You can also see some overview slides we put together [here](https://docs.google.com/presentation/d/1bDGewzIk8mSybwBVwb2kBwPg0WsPwgjgtqJ3_RjKrIg/edit?usp=sharing) (slides strike again!)
