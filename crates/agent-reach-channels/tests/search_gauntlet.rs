//! Search gauntlet — regression benchmark for agent-reach-rs
//!
//! Measures recall@10 and zero-result counts across 16 golden queries.
//! Run: `cargo test --test search_gauntlet`

use agent_reach_channels::exa::ExaChannel;
use agent_reach_channels::github::GitHubChannel;
use agent_reach_core::channel::Channel;
use agent_reach_core::Config;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestCase {
    id: u32,
    query: String,
    target: String,
}

/// What a single channel/query probe actually established.
///
/// The third arm is the one that matters. A free endpoint that answers
/// `429`/`202` told us nothing about whether it can find the target, and
/// scoring that as a miss manufactures a capability problem out of a rate
/// limit — which is exactly how a throttled run once got read as "semantic
/// search is inadequate". Unmeasured probes leave the denominator.
#[derive(Debug, PartialEq)]
enum Outcome {
    Found,
    Miss,
    Unmeasured(String),
}

#[derive(Debug, Default)]
struct Metrics {
    recall_at_10: usize,
    zero_results: usize,
    unmeasured: usize,
    total: usize,
}

impl Metrics {
    fn measured(&self) -> usize {
        self.total.saturating_sub(self.unmeasured)
    }

    fn recall_percent(&self) -> f64 {
        if self.measured() == 0 {
            0.0
        } else {
            (self.recall_at_10 as f64 / self.measured() as f64) * 100.0
        }
    }
}

/// Network-bound and rate-limit sensitive, so it stays out of `cargo test`.
/// Run it deliberately:
/// `cargo test --test search_gauntlet -- --ignored --nocapture`
#[tokio::test]
#[ignore]
async fn search_gauntlet() {
    let golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden_search.json");

    let golden_json = std::fs::read_to_string(&golden_path).expect("golden_search.json must exist");

    let test_cases: Vec<TestCase> =
        serde_json::from_str(&golden_json).expect("golden_search.json must be valid JSON");

    assert_eq!(
        test_cases.len(),
        24,
        "Golden dataset must contain exactly 24 test cases"
    );

    let config = Config::default();
    let gh_channel = GitHubChannel::new();
    let exa_channel = ExaChannel::new();

    let mut github_metrics = Metrics::default();
    let mut exa_metrics = Metrics::default();
    let mut combined_metrics = Metrics::default();

    println!("\n=== Search Gauntlet (24 test cases) ===\n");

    for (i, case) in test_cases.iter().enumerate() {
        // Pace the run. Firing 16 queries back to back at a free public endpoint
        // earns HTTP 429, and a rate-limited run reads exactly like a search
        // engine that cannot find anything — that misreading is what sent the
        // previous round chasing a ranking problem it did not have.
        if i > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        }
        github_metrics.total += 1;
        exa_metrics.total += 1;
        combined_metrics.total += 1;

        println!("#{} | Query: \"{}\"", case.id, case.query);
        println!("     Target: {}", case.target);

        let gh = test_github_channel(&gh_channel, &config, &case.query, &case.target).await;
        record("GitHub", &gh, &mut github_metrics);

        // Second engine: keyless Exa
        let exa = test_exa_channel(&exa_channel, &config, &case.query, &case.target).await;
        record("Exa", &exa, &mut exa_metrics);

        // Combined: found if either engine found it. Only unmeasurable when
        // *neither* engine could be measured — one working engine is a verdict.
        if gh == Outcome::Found || exa == Outcome::Found {
            combined_metrics.recall_at_10 += 1;
        } else if matches!(gh, Outcome::Unmeasured(_)) && matches!(exa, Outcome::Unmeasured(_)) {
            combined_metrics.unmeasured += 1;
            println!("     — NOT MEASURED (both engines throttled)");
        } else {
            combined_metrics.zero_results += 1;
            println!("     ⚠ ZERO RESULTS (no engine found it)");
        }

        println!();
    }

    // Print summary
    println!("=== Final Metrics ===");
    println!(
        "GitHub recall@10:   {}/{} measured ({:.1}%)",
        github_metrics.recall_at_10,
        github_metrics.measured(),
        github_metrics.recall_percent()
    );
    println!(
        "Exa recall@10:      {}/{} measured ({:.1}%)",
        exa_metrics.recall_at_10,
        exa_metrics.measured(),
        exa_metrics.recall_percent()
    );
    println!(
        "Combined recall@10: {}/{} measured ({:.1}%)",
        combined_metrics.recall_at_10,
        combined_metrics.measured(),
        combined_metrics.recall_percent()
    );
    println!(
        "Zero-result queries: {}/{}",
        combined_metrics.zero_results, combined_metrics.total
    );
    println!(
        "Not measured (throttled): github {} · exa {} · combined {}",
        github_metrics.unmeasured, exa_metrics.unmeasured, combined_metrics.unmeasured
    );

    // A run that could not measure most of the set proves nothing either way.
    // Fail loudly rather than reporting a number built on four data points.
    assert!(
        combined_metrics.measured() * 2 >= combined_metrics.total,
        "Only {}/{} queries could be measured — rerun when the endpoints are not throttling",
        combined_metrics.measured(),
        combined_metrics.total
    );

    // Acceptance criteria live in harness/kabul.json, not here.
    //
    // They used to be literals in this file, next to the golden set. The set
    // legitimately grew from 16 cases to 24 — and the same commit that carried
    // that growth also deleted the zero-result assertion and left the recall
    // threshold at an absolute 15, which quietly moved the bar from 94% to 62%.
    // A ratio cannot be diluted by a larger denominator, and a criterion kept
    // in its own file has no legitimate reason to change while a round is
    // being scored.
    let kabul: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harness/kabul.json"),
        )
        .expect("harness/kabul.json must exist"),
    )
    .expect("harness/kabul.json must be valid JSON");

    let min_ratio = kabul["min_recall_ratio"]
        .as_f64()
        .expect("min_recall_ratio");
    let max_zero = kabul["max_zero_results"]
        .as_u64()
        .expect("max_zero_results") as usize;

    // Zero-result first: it is the harder failure. A caller cannot tell an
    // empty list from "does not exist", so one of these is a lie told with
    // confidence — regardless of what the recall column says.
    assert!(
        combined_metrics.zero_results <= max_zero,
        "Zero-result queries must be ≤ {} (got {}/{})",
        max_zero,
        combined_metrics.zero_results,
        combined_metrics.total
    );

    let ratio = combined_metrics.recall_at_10 as f64 / combined_metrics.measured() as f64;
    assert!(
        ratio >= min_ratio,
        "Combined recall must be ≥ {:.0}% (got {:.1}% — {}/{} measured)",
        min_ratio * 100.0,
        ratio * 100.0,
        combined_metrics.recall_at_10,
        combined_metrics.measured()
    );
}

/// Print one probe and fold it into that channel's metrics.
fn record(label: &str, outcome: &Outcome, metrics: &mut Metrics) {
    match outcome {
        Outcome::Found => {
            metrics.recall_at_10 += 1;
            println!("     ✓ {label}: FOUND");
        }
        Outcome::Miss => println!("     ✗ {label}: MISS"),
        Outcome::Unmeasured(why) => {
            metrics.unmeasured += 1;
            println!("     — {label}: NOT MEASURED ({why})");
        }
    }
}

/// A transport refusal is not an answer about relevance.
fn is_throttle(err: &str) -> bool {
    err.contains("429") || err.contains("202") || err.contains("throttled")
}

/// Test GitHub channel via library backend (3-stage relaxation)
async fn test_github_channel(
    channel: &GitHubChannel,
    config: &Config,
    query: &str,
    target: &str,
) -> Outcome {
    let result = channel
        .execute("search", &[query.to_string()], config)
        .await;

    let output = match result {
        Ok(out) => out,
        Err(e) => {
            let msg = e.to_string();
            return if is_throttle(&msg) {
                Outcome::Unmeasured(msg)
            } else {
                eprintln!("      [GitHub channel error: {msg}]");
                Outcome::Miss
            };
        }
    };

    // DEBUG: print raw GitHub response
    if std::env::var("DEBUG_GITHUB").is_ok() {
        eprintln!(
            "      [GitHub raw]: {}",
            serde_json::to_string_pretty(&output.data).unwrap_or_default()
        );
    }

    let empty = vec![];
    let items = output.data.as_array().unwrap_or(&empty);

    // Check top 10 results
    for item in items.iter().take(10) {
        // fullName field (e.g., "cursor/minisqlite")
        if let Some(full_name) = item.get("fullName").and_then(|v| v.as_str()) {
            if full_name.eq_ignore_ascii_case(target) {
                return Outcome::Found;
            }
        }

        // URL fallback
        if let Some(url) = item.get("url").and_then(|v| v.as_str()) {
            if let Some(slug) = extract_repo_slug(url) {
                if slug.eq_ignore_ascii_case(target) {
                    return Outcome::Found;
                }
            }
        }
    }

    Outcome::Miss
}

/// Test the Exa channel — the keyless second engine that actually answers.
///
/// DuckDuckGo's HTML endpoint was scored here for one round and returned 0/16.
/// The cause was measured, not guessed: it replies `202` with a challenge page
/// carrying no result markup, and three paced retries do not clear it. Getting
/// past that means presenting as a browser, which this project does not do.
/// Exa's public MCP endpoint is keyless, answers honestly, and only needs the
/// run to be paced — so it is the fusion partner the benchmark should measure.
async fn test_exa_channel(
    channel: &ExaChannel,
    config: &Config,
    query: &str,
    target: &str,
) -> Outcome {
    let result = channel
        .execute("search", &[query.to_string(), "10".to_string()], config)
        .await;

    let output = match result {
        Ok(out) => out,
        Err(e) => {
            let msg = e.to_string();
            return if is_throttle(&msg) {
                Outcome::Unmeasured(msg)
            } else {
                eprintln!("      [Exa channel error: {msg}]");
                Outcome::Miss
            };
        }
    };

    // Exa returns prose with URLs embedded; match the slug anywhere in it.
    let haystack = output.data.to_string().to_lowercase();
    if haystack.contains(&format!("github.com/{}", target.to_lowercase())) {
        Outcome::Found
    } else {
        Outcome::Miss
    }
}

/// Extract `owner/repo` slug from GitHub URL
fn extract_repo_slug(text: &str) -> Option<String> {
    if let Some(rest) = text.strip_prefix("https://github.com/") {
        Some(rest.trim_end_matches('/').to_string())
    } else if let Some(rest) = text.strip_prefix("http://github.com/") {
        Some(rest.trim_end_matches('/').to_string())
    } else if text.contains('/') && !text.contains("://") {
        Some(text.trim_end_matches('/').to_string())
    } else {
        None
    }
}
