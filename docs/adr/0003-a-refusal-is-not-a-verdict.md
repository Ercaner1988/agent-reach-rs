# 0003 — A transport refusal is not a capability verdict

**Status:** accepted · **Date:** 2026-08-18

## Context

Every keyless search endpoint this project can reach is rate limited, and the
project does not forge browser signatures, rotate proxies, or solve challenges
to get around that. Those limits are therefore permanent facts of the
environment, not obstacles to be engineered away.

The measurement harness fired sixteen queries across two channels with no
pacing. Exa answered `429`. The harness scored each refusal as "did not find
it", reported 0/16, and that zero was read as a statement about search quality —
it became the evidence for a proposed four-dimensional semantic graph.

Two days earlier the same endpoint had measured 7/8. No search engine loses
88 points overnight; the number was never about search.

The failure is structural, not careless. A harness that has only "found" and
"not found" *must* file a refusal under one of them, and "not found" is the one
it will pick. The author of the next round inherits a number that looks like
evidence.

## Decision

A probe yields one of three outcomes: `Found`, `Miss`, `Unmeasured(reason)`.

`429`, `202`, and timeouts produce `Unmeasured`. Unmeasured probes leave the
denominator — recall is reported over what was measured, never over the nominal
set size. A run that cannot measure at least half its set fails outright rather
than publishing a number.

## Consequences

The distinction has to be carried, not merely remembered. It lives in the type,
so a caller cannot collapse a refusal into a miss without writing the collapse
down.

Reported numbers get smaller and more honest: "10/16 measured" states its own
confidence in a way "10/16" does not.

Runs can now fail for being uninformative, which is the intended behaviour and
will occasionally be inconvenient.

The recorded-response layer stores failures with their status codes, so the
unmeasured path can be exercised on demand instead of waiting for a live
endpoint to refuse.

This is deliberately narrow: it says nothing about *why* an endpoint refused, and
it does not make refusals rarer. Pacing does that. This only stops a refusal
from being mistaken for an answer.
