---
layout: post
title: "Safety-Critical Software: The only way out is through"
date: 2026-11-02
---

# Safety-Critical Software: The only way out is through

I have worked in the Automotive industry since 2013. After a few years I felt there must be a better way to create the
software we develop and put on cars. I have great friends that work with and on C and C++; appreciate what these languages
are doing to increase the level of safety and security they have.

I thought there must be something better! There was: Rust. It was really coming up as a darling and well-loved language for
other domains and eventually made it to even the Linux kernel and to hyperscalars. But in 2016 or 2017 when I found Rust it
was still "new" from the Automotive perspective. That's no longer the case. Rust is here in Automotive. Toyota has shipped
600k lines of Rust code into production in the infotainment. Volvo Cars has used Rust for a low power control unit and now
for battery controls. Ampere / Renault are making use. It's here.

I thought I would only be able to see one revolution through the Automotive industry: the adoption of a new programming language.

I was wrong. AI / LLMs / agents have gotten good very quickly. I hold one Claude subscription at Ultra tier ($200/mo), two ChatGPT
subs at Pro 200 tier ($200/mo), and one through the OpenAI Codex for OSS that's Pro 100 tier.

I ran out of all of my subs this week, then created another [OpenAI Pro 200 tier sub][another-open-ai-sub], after
[Anthropic restricted][claude-restricted] the second account I tried to make.

another-open-ai-sub: https://x.com/AwkwardMap/status/2117330681797988773
claude-restricted: https://x.com/AwkwardMap/status/2117273765797360030

Let me walk you through what I'm using all these tokens on and let you be the judge of where regulated industries like Automotive
may be heading.

## Safety standards matter and -- can support using new technologies

Automotive and other regulated industries, e.g. medical devices, avionics, have long-standing ways of doing things. Those
long-standing ways of doing things matter. The standards like ISO 26262 and IEC 61508 and DO-178C are very literally at
times written in blood.

And -- much like how C++ was adopted in developing software in regulated industries, and now Rust is burgeoning on the space,
particularly for middle to high levels of safety-criticaly -- we should look at some use cases and evaluate what we're doing
with AI.

At times, I have certainly pined for having "Rust" be written into the next edition of ISO 26262, but the fact that the standards
are written more generally does help us carefully evaluate new methods.

There's even the argument that I've heard made by an OEM, captured here from the Rust Vision doc work I did, that:

> "What if C, C++ are not considered 'state of the art' anymore? We should have a back-up plan [by starting to use Rust]."

And when coupled with laws in certain jurisdictions could put OEMs, suppliers, and others on a challenging footing as
Rust usage continues to pick up -- and we see the adoption of AI, which Rust works exceptionally well with.

## 

{about Rust}

{here's where AI changes the picture}
