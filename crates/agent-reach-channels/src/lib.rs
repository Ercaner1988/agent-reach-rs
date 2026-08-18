//! Agent Reach Channels — platform-specific readers
//!
//! Each channel module implements the `Channel` trait and orchestrates
//! multiple backends (CLI, API, browser automation) with fallback logic.

pub mod bilibili;
pub mod duckduckgo;
pub mod exa;
pub mod github;
pub mod linkedin;
pub mod reddit;
pub mod rss;
pub mod twitter;
pub mod v2ex;
pub mod web;
pub mod xiaohongshu;
pub mod xiaoyuzhou;
pub mod xueqiu;
pub mod youtube;

pub use bilibili::BilibiliChannel;
pub use duckduckgo::DuckDuckGoChannel;
pub use exa::ExaChannel;
pub use github::GitHubChannel;
pub use linkedin::LinkedinChannel;
pub use reddit::RedditChannel;
pub use rss::RssChannel;
pub use twitter::TwitterChannel;
pub use v2ex::V2exChannel;
pub use web::WebChannel;
pub use xiaohongshu::XiaohongshuChannel;
pub use xiaoyuzhou::XiaoyuzhouChannel;
pub use xueqiu::XueqiuChannel;
pub use youtube::YouTubeChannel;
