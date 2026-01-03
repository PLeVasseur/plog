---
layout: post
title: "Using Spaced-Repetition Systems (SRS) for Learning the FLS Glossary"
date: 2025-11-13
---

# Using Spaced-Repetition Systems (SRS) for Learning the FLS Glossary

tl;dr:
- I'm the team lead of the newly formed [FLS Team] in the Rust Project
- I've been using the program [Anki] to
learn all the items of the [FLS Glossary] for the last 8 days
(it's currently 2025-11-14).
    - You can read an [earlier article] to learn more about my
    past usage of SRS

## What

I'm using an Anki deck with two sub-decks.

- `fls-glossary`
    - `cloze-deletion`
    - `front-back`

Here is that [Anki deck] as an `.apkg`.

Each deck contains 777 cards, one card corresponding to each item
in the [FLS Glossary].

`cloze-deletion`: contains [cloze-deletion] style cards.
`front-back`: contains classic front: term, back: definition style cards.

## How

I've set each subdeck to show me 5 new cards per day, and am working
through the deck in alphabetical order.

## Why

Having both sub-decks allows for the ability to reinforce the knowledge
in different ways.

The choice for 10 total new cards per day is based on simulator output
in Anki which estimates that I could get up to near 100 cards per day
in review at some point. Any more cards than 10 per day and this becomes
fairly dire.

[Anki]: https://apps.ankiweb.net/
[FLS Glossary]: https://rust-lang.github.io/fls/glossary.html
[FLS Team]: https://rust-lang.org/governance/teams/lang/#team-fls
[earlier article]: https://petelevasseur.com/articles/021-power-of-spaced-repetition.html
[Anki deck]: ./022-srs-for-fls-glossary/fls-glossary.apkg
[cloze-deletion]: https://docs.ankiweb.net/editing.html#cloze-deletion
