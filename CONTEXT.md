# Context — agent-reach-rs

The language this project speaks. Glossary only: no implementation detail, no
decisions, no roadmap. Decisions live in `docs/adr/`.

---

## Channel

A platform agent-reach can read — `github`, `exa`, `rss`, `youtube`. A channel
owns an *action* vocabulary (`search`, `repo`, `fetch`) and delegates the work to
its backends.

Not the same thing as a **backend**. `github` is one channel with two backends
(`gh-cli`, `github-api`); `exa` is one channel with two (`exa-api`, `exa-mcp`).
Callers name channels; only the channel names backends.

## Backend

One concrete way to satisfy a channel's action: a subprocess, an HTTP API, a
scraped page. Backends within a channel are ordered — first choice, then
fallback — and each reports its own **availability**.

## Availability

Whether a backend *could* run, asked before it is asked to do anything:
`Available`, `RequiresConfig { missing }`, `NotInstalled { command }`,
`Unavailable { reason }`.

Availability is about configuration, never about results. A backend can be
available and still find nothing.

## Rung · Ladder

A **rung** is one attempt at a search: a query string, optionally a language
filter and a sort order. The **ladder** is the ordered set of rungs a channel
tries for a single user query — the untouched query first, then progressively
relaxed forms.

The ladder exists because the underlying search ANDs its terms: one word the
target does not carry returns nothing at all. Relaxing means removing
*grammatical function words* and moving a language name into a filter. It never
means removing words that carry topic meaning.

## Golden set

The fixed, pre-registered list of (query, target) pairs the search layer is
measured against. Written before the implementation that will be scored on it,
and committed on its own.

Pre-registration is what makes the number mean anything: a set written after
seeing results measures the writer, not the tool.

## Miss ↔ Not measured

**The distinction this project turns on.**

A **miss** is a verdict: the engine answered, and the target was not in the
answer. A **not measured** is the absence of a verdict: the endpoint refused —
`429`, `202`, a timeout — and told us nothing about relevance.

Scoring a refusal as a miss manufactures a capability problem out of a rate
limit. Not-measured probes leave the denominator; a run that cannot measure most
of its set publishes no number at all.

## Referee

The files that decide the score: the golden set, the gauntlet runner, the gate
script. Restored from a fixed git ref immediately before every scoring run, so
whoever is being scored cannot move the threshold they are scored against.

## Gate

A check that costs nothing and runs on every iteration: build, lint, unit tests,
formatting, and the search for golden-set text inside source files. Distinct
from the **gauntlet**, which costs network and runs rarely.

## Gauntlet

The live measurement: every golden query through the real channels, reported as
recall and zero-result counts over the *measured* subset.

## Cassette

Recorded endpoint responses, replayed so the inner development loop needs no
network. Failures are recorded too — a stored `429` is how the not-measured path
is exercised without waiting for a real refusal.

## Shadow mode

Running an unproven ranker alongside the established one, its output collected
and scored but never shown, until it earns the front.

## Mind map

The learned term-to-term expansion layer: an edge carries a weight that rises
when it helps a search and decays when unused. Distinct from the ladder, which
is fixed rules applied to the query text.
