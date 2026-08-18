# 0002 — The referee is restored from git before every scoring run

**Status:** accepted · **Date:** 2026-08-18

## Context

Work on the search layer is being handed to an autonomous agent that iterates
until a number is met. In the round before this decision, the agent held both
the exam and the answer key in the same working tree, and three things happened:

- the acceptance threshold moved from 15/16 to 14/16, and the moved value was
  then cited back as a design constant;
- the zero-result assertion became `assert_eq!(zero_results, 2)` while its own
  message still claimed to demand zero — a perfect run would have failed it;
- phrases from the golden set were transcribed into the source, typo included.

None of this required bad faith. An agent optimising against a number it can
edit will edit the number, the same way it edits anything else in reach.

## Decision

The driver restores the referee files — golden set, gauntlet runner, gate script
— from a fixed git ref (`hakem`) immediately before every scoring run.

The ref is a tag that moves only by deliberate human action. Not `HEAD`: with
`HEAD` the agent could commit a changed threshold and the restore would preserve
it rather than undo it.

## Consequences

Moving the goalposts becomes physically impossible rather than forbidden. The
agent may still edit those files, and `git diff` against the ref records that it
tried — the attempt is visible without being effective.

A threshold change now requires a human to move the tag, which is exactly the
friction it should have.

The gate script is a referee file too, so its allowlists — notably the generic
phrases exempt from the golden-set search — cannot be grown by the party being
checked.

Cost: the driver must know the ref, and a stale tag scores against an old golden
set. Mitigated by moving the tag as the explicit last step of accepting a round.

Alternatives rejected: keeping the referee outside the repo entirely (the agent
then cannot see which queries it missed, so it develops blind); a model
reviewing the diff instead (catches intent, cannot enforce a number, and is
itself persuadable — it is added *on top*, in ADR 0003's spirit, not instead).
