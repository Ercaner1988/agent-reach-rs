# MCP stdio JSON-RPC

`agent-reach-mcp` is a line-delimited stdio JSON-RPC server for MCP tool integration.

## Tools

- `web_read` — read a web page through the Agent Reach web channel.
- `rss_fetch` — fetch and parse an RSS 2.0 or Atom feed URL.
- `rss_parse` — parse RSS 2.0 or Atom XML supplied as text.
- `exa_search` — search through Exa API; requires `exa_api_key` in Agent Reach config.
- `agent_reach_execute` — run an action (`search`, `repo`, `timeline`, ...) on a named channel (`github`, `twitter`, `youtube`, `reddit`, `bilibili`, `xiaohongshu`, `linkedin`, `v2ex`, `xueqiu`, `xiaoyuzhou`, `duckduckgo`, `turath`). Channels switched off via `disabled_channels` are rejected with an explicit error.

Notifications (messages without an `id`, such as `notifications/initialized`) are accepted silently and never answered, per JSON-RPC 2.0.

## Smoke test

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | ./target/debug/agent-reach-mcp
```

## Tool call shape

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"rss_parse","arguments":{"xml":"<rss version=\"2.0\"><channel><title>Example</title></channel></rss>"}}}
```
