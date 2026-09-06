# 0004 — A 200 is not a payload verdict

**Status:** accepted · **Date:** 2026-09-06

## Context

[ADR 0003](0003-a-refusal-is-not-a-verdict.md) settled the case where the
transport says no and the harness hears "not found". This is the same confusion
running the other way: the transport says yes and the channel hears "here is
your data".

Every HTTP backend in this project checked `response.status().is_success()` and
then returned the body. Nothing looked at the body. An audit fired all sixteen
channels and three of them reported `"success": true` over a response that
carried nothing:

- `nitter` returned nitter.net's landing page — `<meta name="description"
  content="nitter.net is offline.">` — as a search result.
- `xhs-web` returned Xiaohongshu's signed-out login wall, sixty kilobytes of
  minified JavaScript with no note in it, as a search result.
- `xiaoyuzhou-web` returned the SPA's 找不到了 page, its 404, as a podcast.

The last one did real damage beyond the log line. Two agents concluded the
channel was dead and wrote that into a report; the channel worked, and answered
correctly the moment it was handed a podcast id that exists. A false success is
worse than a failure because it is quiet: the caller stores it, cites it, and
builds on it.

The pattern is not exotic. A site that serves its own error page over 200 is
the normal shape of a modern SPA, a rate limiter, and every login wall. The
`duckduckgo` backend already knew this and checked for `result__a` in the markup
before believing its own response — that check was written locally and never
generalised, so the other fifteen channels each got to rediscover the problem.

## Decision

A channel's HTTP backend does not return a body until the body has been shown to
carry the thing the action asked for.

`agent_reach_core::backend::require_payload(backend, body, markers)` takes the
markers that a real payload for that channel must contain and fails the backend
when none of them are present. Every HTTP backend that returns scraped or
semi-structured content calls it before its `Ok`.

The check is phrased as **must contain**, never as *must not look like an error
page*. A junk blocklist has to anticipate every way an upstream can fail — a new
maintenance page, a new consent interstitial, a new challenge — and each miss
becomes another silent success. A payload marker is fixed by the action instead:
a search that comes back with no result in it is not a search result, and why it
lacks one is the upstream's business.

Markers are chosen against captured bodies, not from memory. A candidate must be
shown absent from a real junk response and, where one can be obtained, present
in a real good response. Substring collisions are the trap that matters: `noteId`
looked like a sound marker for Xiaohongshu and silently passed the login wall,
because the wall's hydration blob ships an empty `unreadEndNoteId` field.

## Consequences

The three known false successes now fail, with a message that names the missing
markers and the body size, so the next reader can tell a login wall from a
layout change without re-running anything.

Channels can now fail on a 200. This is the point, and it will occasionally be
inconvenient: a marker that goes stale when an upstream renames a field turns a
working channel red. That is the correct direction to fail — a red channel gets
looked at, a green one carrying junk does not.

`duckduckgo` keeps its own hand-rolled check rather than adopting the helper. It
distinguishes *throttled* from *layout changed* and reports which; folding that
into the generic message would trade information for uniformity.

This says nothing about channels whose backends return parsed JSON they already
validate, and nothing about *why* a body is empty. It only stops an error page
from being mistaken for an answer, which is the mirror of what 0003 stopped.
