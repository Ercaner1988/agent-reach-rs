# 0001 — Persistence: turso, not minisqlite

**Status:** superseded in part · **Date:** 2026-08-18 · **Amended:** 2026-08-18

## Context

The planned mind-map layer needs embedded storage. The project constraint is a
pure-Rust dependency chain: no C, no CGO, so cross-compilation stays simple and
the single-language goal holds.

`cursor/minisqlite` was the intended choice and the obvious one — a from-scratch
SQLite reimplementation in Rust, cloned locally, architecturally excellent.

## Decision

Use `turso = "0.7.2"` (tursodatabase/turso, MIT). Keep the minisqlite clone as a
reading text, not a dependency.

## Consequences

**Why not minisqlite — two findings, either one decisive:**

1. **It has no license.** Verified on the local clone: no `LICENSE` file, no
   `license` field in `Cargo.toml`, no license section in the README. Unlicensed
   code is all-rights-reserved; it cannot be depended on, vendored, or forked.
   `publish = false` and `version = "0.0.0"` confirm it was never meant to be
   consumed. The name `minisqlite` on crates.io belongs to an unrelated project.

2. **No FTS5.** `crates/minisqlite-sql/src/parser/ddl.rs:43` rejects
   `CREATE VIRTUAL TABLE` outright. The schema drafted for the mind map used
   `USING fts5`, and the search pattern being ported rests on FTS5 + BM25.

Its 201,681 lines are not a small easy dependency — but the internal seams are
clean and it is worth reading as a Rust storage engine. It stays in the folder.

**Why turso:** the only licensed pure-Rust SQLite. MIT, 23.9k stars, upstream
pushed the day this was written, stable line at `0.7.2` with 730k downloads,
full-text search available.

> **Amendment — this decision was wrong on its central claim.**
>
> "Pure Rust" was taken from the project's own description and never checked
> against the dependency tree. It should have been:
>
> ```
> cargo tree | grep -E "sys$|^cc "     → libmimalloc-sys, zstd-sys, cc
> cargo tree -i cc                     → aegis → turso_core → turso
> ```
>
> `turso 0.7.2` resolves to 595 dependency lines and pulls `cc` as a build
> dependency, surviving `--no-default-features`. It does **not** satisfy the
> project's zero-C constraint, which was the whole reason it was chosen over
> the alternatives.
>
> El-Kassâm found this while working ticket C and declined to add the crate.
> The named cause was wrong — `libsql` appears zero times in the tree — but the
> conclusion was right, and the ticket's own rule 9 ("measurement must earn the
> dimension") made declining correct on independent grounds.
>
> Everything above about `cursor/minisqlite` still holds: no license is still
> disqualifying, and `CREATE VIRTUAL TABLE` is still rejected outright.
>
> **Open, not decided.** If a storage layer is ever earned by a measurement,
> the candidates must be re-examined with `cargo tree -i cc` first. `redb`
> (pure Rust, MIT/Apache, stable v4, no SQL) is the obvious next one to check.

**No fork.** 1,268 forks exist; the most-starred is the maintainer's own working
copy at 19 stars, and the rest are single-digit or stale copies from the
project's former `limbo` name. Forks of an active project are PR staging, not
alternative distributions.

**Not yet added.** Rounds A and B do not touch storage. The dependency lands
when the measurement shows the layer is needed — not before.
