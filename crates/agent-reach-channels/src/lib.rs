//! Agent Reach Channels — platform-specific readers
//!
//! Each channel module implements the `Channel` trait and orchestrates
//! multiple backends (CLI, API, browser automation) with fallback logic.

pub mod bilibili;
pub mod github;
pub mod reddit;
pub mod rss;
pub mod twitter;
pub mod web;
pub mod xiaohongshu;
pub mod youtube;
// Future channels (to be implemented):
// pub mod linkedin;
// pub mod v2ex;
// pub mod xueqiu;
// pub mod xiaoyuzhou;
// pub mod exa_search;

pub use bilibili::BilibiliChannel;
pub use github::GitHubChannel;
pub use reddit::RedditChannel;
pub use rss::RssChannel;
pub use twitter::TwitterChannel;
pub use web::WebChannel;
pub use xiaohongshu::XiaohongshuChannel;
pub use youtube::YouTubeChannel;
