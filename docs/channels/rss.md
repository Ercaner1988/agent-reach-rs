# RSS Kanalı

RSS 2.0 ve Atom besleme okuyucu/çözümleyici kanal.

## Eylemler

| Eylem | Açıklama | Argümanlar |
|-------|----------|-----------|
| `fetch` | Besleme URL'sini indirir ve çözümler | `[url]` |
| `parse` | Verilen XML'i çözümler | `[xml]` |

## Arka-uçlar

| Arka-uç | Açıklama | Kullanılabilirlik |
|---------|----------|------------------|
| `rss-parser` | RSS 2.0 + Atom çözümleme (yerleşik) | Her zaman |

## Kullanım

### Görev JSON

```json
[
  {
    "id": "rss-fetch-1",
    "channel": "rss",
    "action": "fetch",
    "args": ["https://blog.rust-lang.org/feed.xml"]
  },
  {
    "id": "rss-parse-1",
    "channel": "rss",
    "action": "parse",
    "args": ["<?xml version=\"1.0\"?><rss version=\"2.0\"><channel>...</channel></rss>"]
  }
]
```

### Komut

```bash
agent-reach execute --task-file rss_tasks.json --output rss_log.json --verbose
```

### Çıktı

```json
{
  "task_id": "rss-fetch-1",
  "success": true,
  "channel": "rss",
  "backend": "rss-parser",
  "duration_ms": 575,
  "output": {
    "format": "rss",
    "feed": {
      "title": "Rust Blog",
      "link": "https://blog.rust-lang.org/",
      "description": "..."
    },
    "items": [
      {
        "title": "Enabling the next iteration of the borrow checker on nightly",
        "url": "https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly/",
        "description": "...",
        "published_at": "Tue, 04 Aug 2026 00:00:00 GMT"
      }
    ]
  }
}
```

## Notlar

- `parse` eylemi yerleşik RSS 2.0 → Atom sırasıyla dener
- `fetch` eylemi yapılandırılmış vekili (proxy) kullanır
- Atom çıktısında `published_at` RFC 3339 biçimindedir
