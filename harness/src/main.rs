//! Gates and loop driver for agent-reach-rs.
//!
//! Two commands:
//!
//! ```text
//! harness gates                 the seven free checks — no network, seconds
//! harness run --ticket A        one round: agent, gates, gauntlet, review, stop
//! ```
//!
//! This replaces a pair of PowerShell scripts. They worked, but they put a
//! second language beside a project whose point is to have one, and they only
//! ran on Windows. Nothing here is new behaviour; it is the same seven gates and
//! the same loop, in the language of the thing it guards.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const REFEREE: &[&str] = &["harness/kabul.json"];

const GOLDEN: &str = "crates/agent-reach-channels/tests/golden_search.json";
const GAUNTLET: &str = "crates/agent-reach-channels/tests/search_gauntlet.rs";
const CRITERIA: &str = "harness/kabul.json";

/// Symbols proving the runner is still wired to the criteria file. Deleting an
/// assertion is the quietest way to move a threshold; this makes it the loudest.
///
/// A tripwire, not a proof: it is string matching, so a determined rewrite can
/// step over it. It catches the realistic cases — an assertion turned into a
/// `println!`, or a comparison reworded past the symbol — and it costs nothing.
/// The reviewer in part 2 is what covers intent.
const CRITERIA_SYMBOLS: &[&str] = &[
    "kabul.json",
    "min_recall_ratio",
    "max_zero_results",
    "zero_results <= max_zero",
];

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = repo_root();

    let code = match args.first().map(String::as_str) {
        Some("gates") => gates(&root),
        Some("run") => run_round(&root, &args[1..]),
        _ => {
            eprintln!(
                "usage:\n  \
                 harness gates\n  \
                 harness run --ticket <A|B|C> [--referee <ref>] [--model <m>] \\\n    \
                 [--session <id>] [--reviewer <provider>] [--reviewer-model <m>] \\\n    \
                 [--dry-run]"
            );
            2
        }
    };
    std::process::ExitCode::from(code)
}

/// The repository root: the parent of the directory holding this crate.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("harness/ always has a parent")
        .to_path_buf()
}

// ── gates ────────────────────────────────────────────────────────────────────

fn gates(root: &Path) -> u8 {
    let cargo_checks: [(&str, &[&str]); 4] = [
        ("build", &["build", "--workspace"]),
        (
            "clippy",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        ),
        ("unit tests", &["test", "--workspace"]),
        ("formatting", &["fmt", "--check"]),
    ];

    for (label, argv) in cargo_checks {
        eprintln!("-- {label}");
        let out = Command::new("cargo")
            .args(argv)
            .current_dir(root)
            .stdin(Stdio::null())
            .output();
        match out {
            Ok(o) if o.status.success() => eprintln!("   green"),
            Ok(o) => {
                eprintln!("   RED: {label}");
                let text = String::from_utf8_lossy(&o.stderr);
                for line in text.lines().rev().take(12).collect::<Vec<_>>().iter().rev() {
                    eprintln!("     {line}");
                }
                return 1;
            }
            Err(e) => {
                eprintln!("   RED: could not run cargo {}: {e}", argv[0]);
                return 1;
            }
        }
    }

    if let Err(code) = gate_answer_key(root) {
        return code;
    }
    gate_referee_tamper(root);
    if let Err(code) = gate_criteria_wired(root) {
        return code;
    }

    eprintln!("\nAll gates green.");
    0
}

/// Gate 5 — no text from the answer key may appear in source.
///
/// Two-word phrases, not single words: queries contain ordinary words (`rust`,
/// `python`, `written`) that legitimately appear in language tables and
/// stop-word lists, and matching those accuses correct code. Every violation
/// seen so far was a phrase — including one carrying a typo from the test set,
/// which is how it was proven copied rather than coincidental.
fn gate_answer_key(root: &Path) -> Result<(), u8> {
    eprintln!("-- answer-key grep");

    let golden = match read_json(&root.join(GOLDEN)) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("   RED: {GOLDEN}: {e}");
            return Err(1);
        }
    };
    let criteria = read_json(&root.join(CRITERIA)).unwrap_or(serde_json::Value::Null);
    let generic: HashSet<String> = criteria["generic_bigrams"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .map(str::to_lowercase)
                .collect()
        })
        .unwrap_or_default();

    let mut wanted: HashSet<String> = HashSet::new();
    for case in golden.as_array().into_iter().flatten() {
        if let Some(t) = case["target"].as_str() {
            wanted.insert(t.to_lowercase());
        }
        if let Some(q) = case["query"].as_str() {
            let words: Vec<&str> = q.split_whitespace().collect();
            for pair in words.windows(2) {
                let bigram = format!("{} {}", pair[0], pair[1]).to_lowercase();
                if !generic.contains(&bigram) {
                    wanted.insert(bigram);
                }
            }
        }
    }

    let mut violations = Vec::new();
    for file in rust_sources(&root.join("crates")) {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let lower = text.to_lowercase();
        for phrase in &wanted {
            if lower.contains(phrase) {
                let name = file.file_name().unwrap_or_default().to_string_lossy();
                violations.push(format!("{name}: \"{phrase}\""));
            }
        }
    }

    if !violations.is_empty() {
        eprintln!("   RED: answer-key text found in source");
        violations.sort();
        for v in &violations {
            eprintln!("     {v}");
        }
        eprintln!("   Passing the exam and deleting the exam are different things.");
        return Err(1);
    }
    eprintln!("   green ({} phrases searched)", wanted.len());
    Ok(())
}

/// Gate 6 — report, do not block. The driver restores these before scoring, so
/// an edit cannot change the score; it should still be visible that one was made.
fn gate_referee_tamper(root: &Path) {
    eprintln!("-- referee watch");
    let mut argv = vec!["diff", "--name-only", "--"];
    argv.extend(REFEREE);
    let out = Command::new("git")
        .args(&argv)
        .current_dir(root)
        .output()
        .ok();
    let dirty = out
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    if dirty.is_empty() {
        eprintln!("   green");
    } else {
        eprintln!("   NOTE: referee files modified:");
        for f in dirty.lines() {
            eprintln!("     {f}");
        }
        eprintln!("   They will be restored from git before scoring.");
    }
}

/// Gate 7 — the runner must still consult the criteria file.
fn gate_criteria_wired(root: &Path) -> Result<(), u8> {
    eprintln!("-- criteria wiring");
    let Ok(text) = std::fs::read_to_string(root.join(GAUNTLET)) else {
        eprintln!("   RED: cannot read {GAUNTLET}");
        return Err(1);
    };
    let missing: Vec<&&str> = CRITERIA_SYMBOLS
        .iter()
        .filter(|s| !text.contains(**s))
        .collect();
    if !missing.is_empty() {
        eprintln!("   RED: the gauntlet no longer reads its acceptance criteria. Missing:");
        for m in missing {
            eprintln!("     {m}");
        }
        eprintln!("   Deleting an assertion is not passing a threshold.");
        return Err(1);
    }
    eprintln!("   green");
    Ok(())
}

// ── driver ───────────────────────────────────────────────────────────────────

struct Opts {
    ticket: String,
    referee: String,
    model: Option<String>,
    session: String,
    reviewer: String,
    reviewer_model: String,
    dry_run: bool,
}

fn parse_opts(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts {
        ticket: String::new(),
        referee: "hakem".into(),
        // Left to the session. Two rounds were lost to model-shopping inside
        // the `auto/*` namespace: `auto/best-coding` and `auto/coding:reliable`
        // both resolve to antigravity/gemini-3.6-flash-high, so swapping them
        // changed the label and nothing else. Pinning a model here would also
        // override whatever the resumed session is already working under.
        model: None,
        // The standing agent-reach-rs session. 1067 messages of this project's
        // history, and calling tools in this repository throughout.
        session: "20260817_183532_59ccce".into(),
        // A reviewer has to be a model that did not write the code. The old
        // default was Kervan's Gemini, which is the family the agent runs under
        // — self-review dressed as review. This one is DeepSeek, verified at the
        // provider level in OmniRoute's call log rather than by trusting the
        // catalogue: `oc/deepseek-v4-flash-free` really is served by opencode as
        // deepseek-v4-flash-free, and it is free. The neighbouring
        // `kc/deepseek/deepseek-v4-pro-0813` answered too, but the log shows it
        // returned nothing and the request fell through to Gemini — a right
        // answer from the wrong model, which is the failure this whole harness
        // exists to catch.
        reviewer: "custom:omniroute-hermes-beyin".into(),
        reviewer_model: "oc/deepseek-v4-flash-free".into(),
        dry_run: false,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut next = || it.next().cloned().ok_or(format!("{a} needs a value"));
        match a.as_str() {
            "--ticket" => o.ticket = next()?,
            "--session" => o.session = next()?,
            "--referee" => o.referee = next()?,
            "--model" => o.model = Some(next()?),
            "--reviewer" => o.reviewer = next()?,
            "--reviewer-model" => o.reviewer_model = next()?,
            "--dry-run" => o.dry_run = true,
            other => return Err(format!("unknown flag: {other}")),
        }
    }
    if o.ticket.is_empty() {
        return Err("--ticket is required".into());
    }
    Ok(o)
}

fn run_round(root: &Path, args: &[String]) -> u8 {
    let opts = match parse_opts(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };

    if !git_ref_exists(root, &opts.referee) {
        eprintln!(
            "referee ref not found: {} (pass --referee <ref>)",
            opts.referee
        );
        return 2;
    }
    let round_start = git_head(root);
    recover_stashed_cwd(root);

    section(&format!(
        "Ticket {} · referee {} · start {}",
        opts.ticket,
        opts.referee,
        &round_start[..7.min(round_start.len())]
    ));
    restore_referee(root, &opts.referee);

    let ticket_path = root
        .join("harness/biletler")
        .join(format!("bilet_{}.md", opts.ticket));
    let Ok(ticket) = std::fs::read_to_string(&ticket_path) else {
        eprintln!("no such ticket: {}", ticket_path.display());
        return 2;
    };

    let cassette = root.join("harness/kaset");

    if opts.dry_run {
        eprintln!("\n[dry run] the agent is not called; gates and gauntlet still run.");
    } else {
        section("Agent");
        let prompt = format!(
            "{ticket}\n\n\
             --- HARNESS NOTE ---\n\
             Run this as often as you like in the inner loop; it is free and takes seconds:\n    \
             cargo run --manifest-path harness/Cargo.toml -- gates\n\
             Do not deliver until all seven are green.\n\n\
             AGENT_REACH_CASSETTE is set: search calls replay from the cassette. A new query \
             reaches the network once and is recorded. Do NOT run the live gauntlet yourself — \
             the driver runs it.\n\n\
             Leave the referee files alone; they are restored from git before scoring:\n{}\n\n\
             Start now, with a tool call. A reply that only describes a plan ends the \
             round with an empty diff and is scored as no work done.\n",
            REFEREE.join("\n")
        );
        // The agent's shell does not inherit this process's directory: it starts in
        // whatever `terminal.cwd` says, and that setting outranks both the spawned
        // cwd and the TERMINAL_CWD environment variable (measured, all three). A
        // round once produced a plan and an empty diff because the agent was looking
        // at a different repository. Point the setting at the root and put it back
        // afterwards, including when the round fails. `--worktree` is gone for the
        // same reason: the shell would stay in the root while the worktree it made
        // went unscored.
        let saved_cwd = hermes_cwd();
        stash_cwd(root, saved_cwd.as_deref());
        set_hermes_cwd(&root.display().to_string());

        let mut cmd = Command::new("hermes");
        cmd.arg("-z").arg(&prompt);
        // Resume the standing session rather than opening a fresh one. A new
        // session arrives with none of this project's history and gets routed
        // wherever the table sends it that minute; the standing one already
        // knows the repository and has been calling tools in it for two days.
        if !opts.session.is_empty() {
            cmd.arg("--resume").arg(&opts.session).arg("--in").arg(root);
        }
        if let Some(m) = &opts.model {
            cmd.arg("-m").arg(m);
        }
        let status = cmd
            .current_dir(root)
            .env("AGENT_REACH_CASSETTE", &cassette)
            .status();

        if let Some(prev) = &saved_cwd {
            set_hermes_cwd(prev);
        }
        stash_cwd(root, None);
        if let Ok(s) = status {
            if !s.success() {
                eprintln!("  agent exited non-zero: {s}");
            }
        }
    }

    section("Gates");
    restore_referee(root, &opts.referee);
    if gates(root) != 0 {
        // Red rounds used to stop here. That threw away the round's most useful
        // hour: three consecutive red rounds were scored by the gates as one
        // repeated fault, while the reviewer, given the same diff by hand,
        // found two the gates cannot see — a learning loop with no caller and a
        // "shadow" expansion pushed onto the live search path. The gates say a
        // rule was broken; the reviewer says what else is wrong underneath.
        eprintln!("\nGATE RED. Not an approved round — reviewing it anyway.");
        section("Review");
        let review = review_round(root, &round_start, &opts);
        eprintln!("\n  review   : {review}");
        eprintln!("  The next ticket opens after a human says so.");
        return 1;
    }

    section("Gauntlet (live, at most 2 runs)");
    restore_referee(root, &opts.referee);
    let mut passed = false;
    for attempt in 1..=2 {
        eprintln!("  run {attempt}/2");
        let out = Command::new("cargo")
            .args([
                "test",
                "--test",
                "search_gauntlet",
                "--",
                "--ignored",
                "--nocapture",
            ])
            .current_dir(root)
            .env("AGENT_REACH_CASSETTE", &cassette)
            .output();
        let Ok(out) = out else { break };
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        for line in text.lines().filter(|l| {
            l.contains("recall@10")
                || l.contains("Zero-result")
                || l.contains("Not measured")
                || l.contains("must be")
                || l.contains("could be measured")
        }) {
            eprintln!("    {}", line.trim());
        }
        if out.status.success() {
            passed = true;
            break;
        }
        // A throttled run is not a verdict; that is the only reason to retry.
        if text.contains("could be measured") {
            eprintln!("    throttled — waiting 60s for one more attempt");
            std::thread::sleep(std::time::Duration::from_secs(60));
        } else {
            break;
        }
    }

    section("Review");
    let review = review_round(root, &round_start, &opts);

    section("Round complete");
    eprintln!("  gauntlet : {}", if passed { "GREEN" } else { "RED" });
    eprintln!("  diff     : {}", diff_stat(root, &round_start));
    eprintln!("  review   : {review}");
    if review == "not run" {
        eprintln!(
            "\n  WARNING: the diff was not reviewed. An unreviewed round is not an approved round."
        );
    }
    eprintln!("\n  The next ticket opens after a human says so.");
    u8::from(!passed)
}

/// One adversarial pass over the round's diff, by a model that did not write it.
fn review_round(root: &Path, round_start: &str, opts: &Opts) -> String {
    // Recorded search responses are data, not work: one round carried 107 of
    // them and the diff came to 2.2 MB, nearly all of it engine output nobody
    // wrote. Reviewing that is both useless and impossible to fit in a prompt.
    let exclude = ":(exclude,glob)**/kaset/**";
    let mut diff = capture(
        root,
        "git",
        &["diff", &format!("{round_start}..HEAD"), "--", ".", exclude],
    );
    if diff.trim().is_empty() {
        diff = capture(root, "git", &["diff", "--", ".", exclude]);
    }
    if diff.trim().is_empty() {
        eprintln!("  no changes, review skipped");
        return "no changes".into();
    }

    // Still too large to send is a fact the round should report, not one the
    // reviewer should silently swallow half of.
    const MAX_DIFF: usize = 400_000;
    if diff.len() > MAX_DIFF {
        eprintln!(
            "  diff is {} bytes, over the {MAX_DIFF} the reviewer is given; \
             sending the first {MAX_DIFF} and saying so",
            diff.len()
        );
        diff.truncate(MAX_DIFF);
        diff.push_str("\n\n[TRUNCATED — the diff did not fit. Review what is here and say plainly that the rest was not seen.]");
    }

    let prompt = format!(
        "The diff below is one round of work by an AI agent. Your only job is to find \
         defects; do not look for praise. If you find none, say \"clean\" and stop. \
         Give a ranked list, heaviest first.\n\n\
         Look for these four in particular:\n\
         1. Target fitting — constants derived from the test set, special cases, lists \
         written by reading the answer key.\n\
         2. Scope creep — a file, channel or dependency the ticket did not ask for.\n\
         3. Comment/code contradiction — the comment claims the opposite of what the code does.\n\
         4. A test quietly weakened — a threshold moved, an assertion deleted, a case removed.\n\n\
         --- DIFF ---\n{diff}"
    );
    let prompt_path = root.join("harness/son-denetim-istemi.txt");
    let _ = std::fs::write(&prompt_path, &prompt);

    let status = if opts.reviewer == "dsh" {
        Command::new("dsh")
            .args(["--profile", "headless"])
            .arg(&prompt)
            .current_dir(root)
            .status()
    } else {
        Command::new("hermes")
            .arg("-z")
            .arg(&prompt)
            .args(["--provider", &opts.reviewer, "-m", &opts.reviewer_model])
            .current_dir(root)
            .status()
    };

    match status {
        Ok(s) if s.success() => format!("ran ({}/{})", opts.reviewer, opts.reviewer_model),
        _ => {
            eprintln!(
                "  '{}' could not be called. Is it configured? -> hermes fallback list",
                opts.reviewer
            );
            eprintln!("  prompt written to harness/son-denetim-istemi.txt");
            eprintln!("  Hand it to a model before approving this round.");
            "not run".into()
        }
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn section(title: &str) {
    eprintln!("\n=== {title} ===");
}

fn restore_referee(root: &Path, referee: &str) {
    let tampered = {
        let mut argv = vec!["diff", "--name-only", referee, "--"];
        argv.extend(REFEREE);
        capture(root, "git", &argv)
    };
    if !tampered.trim().is_empty() {
        eprintln!("  referee tampered with, restoring:");
        for f in tampered.lines() {
            eprintln!("    {f}");
        }
    }
    let mut argv = vec!["checkout", referee, "--"];
    argv.extend(REFEREE);
    let _ = Command::new("git").args(&argv).current_dir(root).status();
}

fn git_ref_exists(root: &Path, r: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", r])
        .current_dir(root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn git_head(root: &Path) -> String {
    capture(root, "git", &["rev-parse", "HEAD"])
        .trim()
        .to_string()
}

/// Where the agent's shell will start. `None` if hermes cannot be asked, in
/// which case the round leaves the setting alone rather than guessing at one.
fn hermes_cwd() -> Option<String> {
    let out = Command::new("hermes")
        .args(["config", "get", "terminal.cwd"])
        .output()
        .ok()?;
    let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !value.is_empty() && !value.contains('\n')).then_some(value)
}

fn cwd_stash(root: &Path) -> PathBuf {
    root.join("harness/.onceki_cwd")
}

/// Remember the value to put back, on disk, because a killed round never
/// reaches its own restore — measured: one interrupted round left the setting
/// pointing at this repository, which would silently follow the user into their
/// next unrelated session. `None` clears the note once the restore has happened.
fn stash_cwd(root: &Path, previous: Option<&str>) {
    let path = cwd_stash(root);
    match previous {
        Some(p) => {
            let _ = std::fs::write(path, p);
        }
        None => {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn recover_stashed_cwd(root: &Path) {
    let path = cwd_stash(root);
    let Ok(previous) = std::fs::read_to_string(&path) else {
        return;
    };
    let previous = previous.trim();
    if !previous.is_empty() {
        eprintln!("  a previous round did not finish; putting terminal.cwd back");
        set_hermes_cwd(previous);
    }
    let _ = std::fs::remove_file(path);
}

fn set_hermes_cwd(dir: &str) {
    let ok = Command::new("hermes")
        .args(["config", "set", "terminal.cwd", dir])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !ok {
        eprintln!("  could not set terminal.cwd; the agent may be looking elsewhere");
    }
}

fn diff_stat(root: &Path, from: &str) -> String {
    let s = capture(
        root,
        "git",
        &["diff", "--shortstat", &format!("{from}..HEAD")],
    );
    let s = if s.trim().is_empty() {
        capture(root, "git", &["diff", "--shortstat"])
    } else {
        s
    };
    let s = s.trim().to_string();
    if s.is_empty() {
        "none".into()
    } else {
        s
    }
}

fn capture(root: &Path, program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// Every `.rs` under `dir`, skipping test directories — fixtures there may
/// legitimately name the same repositories the answer key does.
fn rust_sources(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let name = p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if name != "tests" && name != "target" {
                    stack.push(p);
                }
            } else if p.extension().is_some_and(|e| e == "rs") {
                found.push(p);
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opts_default_to_the_measured_reviewer() {
        let o = parse_opts(&["--ticket".into(), "A".into()]).unwrap();
        assert_eq!(o.ticket, "A");
        assert_eq!(o.referee, "hakem");
        assert_eq!(o.reviewer, "custom:omniroute-hermes-beyin");
        assert_eq!(o.reviewer_model, "oc/deepseek-v4-flash-free");
        // The point of the reviewer is that it did not write the code. The agent
        // runs under Gemini, so a Gemini reviewer is self-review with a second
        // name on it — the one default this must never drift back to.
        assert!(!o.reviewer_model.contains("gemini"));
        assert!(!o.dry_run);
    }

    #[test]
    fn opts_reject_a_missing_ticket() {
        assert!(parse_opts(&["--dry-run".into()]).is_err());
        assert!(parse_opts(&["--ticket".into()]).is_err());
        assert!(parse_opts(&["--nope".into()]).is_err());
    }

    #[test]
    fn rust_sources_skips_test_directories() {
        // A fixture under tests/ may name an answer-key repository without that
        // being a violation, so the walk must not reach it.
        let root = repo_root();
        let files = rust_sources(&root.join("crates"));
        assert!(!files.is_empty(), "expected some sources under crates/");
        assert!(
            files
                .iter()
                .all(|p| !p.components().any(|c| c.as_os_str() == "tests")),
            "walk reached a tests/ directory"
        );
    }
}
