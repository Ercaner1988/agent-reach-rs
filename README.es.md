**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **Un motor de lectura puramente en Rust que proporciona a los agentes de IA ojos para ver toda la internet**

`agent-reach-rs` es un espacio de trabajo modular de Rust que permite a los agentes de IA (Hermes, Claude, Codex, OpenCode) leer de manera confiable datos de páginas web, redes sociales, bases de datos legislativas y académicas, y archivos locales de audio/video.

Mediciones en las que se basa este documento: `git rev-parse --short HEAD` → `dbe3b0c`, `cargo test --workspace` → **58 passed, 0 failed, 1 ignored**, `harness/gates` → **8 gates green** (7 September 2026).

---

## 🎯 1. Objetivos y Características Completadas

- **17 channel readers** — cada canal intenta uno o más backends; si uno falla, se intenta el siguiente (escalera CLI → API → scraper).
- **Sin dependencia externa de FFmpeg (`MediaInspector`):** MP3, WAV, AAC, FLAC, OGG y MKV se procesan con `symphonia` 0.5 sin ningún binario `ffmpeg` externo. El campo `format_name` siempre devuelve `symphonia-native`.
- **Interruptor de canales (`disabled_channels`):** Las fuentes no deseadas se pueden apagar a través de la configuración o una variable de entorno. Un canal deshabilitado produce un **error diferente al de un canal roto** — uno es una decisión, el otro es un fallo por investigar.
- **Verificación de payload (`require_payload`, ADR 0004):** Las respuestas que devuelven HTTP 200 sin contenido de payload real (muros de inicio de sesión, avisos sin conexión, páginas 404) se detectan y no se tratan silenciosamente como exitosas.
- **Puerta golden pre-push (`.githooks/pre-push`):** Ejecuta localmente las 8 golden gates antes del push; detiene el push si las comprobaciones de formato o compilación fallan.
- **Puerta de paridad de CI (`gate_ci_parity`):** Verifica que todas las comprobaciones `cargo` definidas en los flujos de trabajo de CI estén presentes en las gates locales de harness.
- **Cassette system:** Las respuestas se graban y se reproducen; los fallos también se graban, por lo que la ruta "not measured" se puede ejercitar sin red. Apagado por defecto.
- **Servidor MCP:** Model Context Protocol 2024-11-05, stdio JSON-RPC, cinco herramientas.
- **Transcripción local (`transcribe`):** Un archivo local de audio/video se transcribe con Whisper (`whisper-large-v3-turbo`) a través de Groq u OpenAI.
- **Referee-protected measurement:** El golden set, los umbrales de aceptación y las comprobaciones contra manipulación residen en archivos separados; quien mide no puede alterar lo medido.

### Canales (17)

| Canal | Acciones | Nota |
| :--- | :--- | :--- |
| `web` | `read` | Texto de la página a través de Jina Reader |
| `rss` | `fetch`, `parse` | RSS 2.0 + Atom; las publicaciones de Substack también utilizan este canal |
| `twitter` | `search`, `timeline`, `thread` | Dos backends |
| `youtube` | `metadata`, `transcript`, `search` | Dos backends |
| `github` | `repo`, `issue`, `pr`, `search` | CLI de `gh` + API REST |
| `reddit` | `subreddit`, `search`, `post` | PRAW + API REST |
| `bilibili` | `video`, `info`, `search` | Dos backends |
| `xiaohongshu` | `note`, `search` | Backend web |
| `linkedin` | `profile`, `company`, `search` | Requiere credenciales |
| `v2ex` | `topic`, `node`, `hot`, `latest` | Open API |
| `xueqiu` | `quote`, `stock`, `timeline`, `search` | Datos de mercado |
| `xiaoyuzhou` | `podcast`, `episode` | Podcasts |
| `exa` | `search` | Búsqueda semántica, requiere una clave |
| `duckduckgo` | `search` | Extractor de HTML, sin clave |
| `turath` | `search`, `book`, `author`, `page` | Obras clásicas islámicas; devuelve **volume and page numbers** |
| `mevzuat` | `mevzuat`, `metin`, `fihrist` | Legislación turca (`mevzuat.gov.tr`), consultada directamente por el número de ley |
| `uinjkt` | `identify`, `journals`, `articles`, `record` | Revistas académicas islámicas de UIN Jakarta (OAI-PMH 2.0, open access) |

### Próximas fuentes en cola

Documentadas con endpoints medidos en `docs/YOL-HARITASI-KAYNAKLAR.md`:
**UYAP Emsal** y **Yargıtay Karar Arama** (ambos JSON sin clave, `aramalist` + `getDokuman`), **Danıştay** (endpoint activo, cuerpo de búsqueda por resolver), **Pew Research** (`robots.txt` explícitamente concede acceso a agentes, `ai-input=yes`), **Lexpera** (muro de suscripción + mandatory 10-second crawl delay → test superado por humanos), además de Wikipedia, Stanford Encyclopedia of Philosophy, TDV Encyclopedia of Islam, OpenAlex, Crossref y el Turkish National Thesis Center.

---

## 🏗️ 2. Arquitectura y Módulos

```text
agent-reach-rs/
├── Cargo.toml                    # Workspace (symphonia, tokio, reqwest, clap)
├── .githooks/                    # Rust-native git hooks (pre-push, post-commit, post-push)
├── crates/
│   ├── agent-reach-core/         # Channel, Backend, Config, Cassette, require_payload,
│   │                             #   MediaInspector, doctor, error
│   ├── agent-reach-channels/     # 17 channel implementations (including mevzuat, uinjkt, turath)
│   ├── agent-reach-mcp/          # MCP JSON-RPC server (stdio)
│   └── agent-reach-cli/          # Command-line client (binary: agent-reach)
├── harness/                      # Referee gates — built separately (outside workspace)
│   ├── kabul.json                # Acceptance thresholds (a file SEPARATE from the golden set)
│   └── biletler/                 # Tickets A · B · C and their outcomes
└── docs/
    ├── adr/                      # Architecture decisions 0001 · 0002 · 0003 · 0004
    └── YOL-HARITASI-KAYNAKLAR.md # Source generations and measured endpoints
```

**Por qué `harness` se encuentra fuera del workspace:** si la herramienta que ejecuta las gates se compilara junto con el árbol de código que evalúa, un error en el código también derribaría al árbitro.

### Tipos principales

| Tipo | Responsabilidad |
| :--- | :--- |
| `Channel` | El nombre, acciones y ejecutor de una fuente |
| `Backend` | Una única ruta bajo un canal; reporta su salud a través de `BackendStatus` |
| `Config` | Lee/escribe `~/.agent-reach/config.yaml`, aplica `channel_enabled` |
| `Cassette` | Grabación y reproducción de respuestas (`AGENT_REACH_CASSETTE`) |
| `MediaInspector` | Analizador multimedia puramente en Rust (`symphonia`) |
| `HealthCheck` | Tipo de datos detrás de la salida de `doctor` |

---

## 🚀 3. Instalación y Configuración

### Prerrequisitos

- **Rust 1.75+** (`cargo` y `rustc`).
- **No se requieren binarios externos:** FFmpeg, Python y Node.js no son obligatorios. Los backends opcionales para ciertos canales (`gh` CLI, `praw`, `BBDown`) se utilizan cuando están instalados; de lo contrario el canal recurre a su otro backend.

### Compilación

```bash
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs
cargo build --release
```

Binario producido: `target/release/agent-reach` (`agent-reach.exe` en Windows).

> **Nota:** Este repositorio no publica binarios precompilados descargables. `dist-workspace.toml` establece `installers = []`; los assets de cada versión (release) se producen solo como archivos comprimidos. Compilar desde el código fuente es el único método.

### Configuración por primera vez

```bash
agent-reach install                      # Creates the ~/.agent-reach directory
agent-reach configure github_token <tk>  # Set a key
agent-reach configure --json             # View current settings
agent-reach doctor                       # Channel health check
agent-reach doctor --json                # Machine-readable health report
```

El comando `install` **no** instala herramientas externas; solo prepara el directorio de configuración.

### Deshabilitar canales

`~/.agent-reach/config.yaml`:

```yaml
disabled_channels:
  - reddit
  - linkedin
```

O, para una única ejecución sin modificar el archivo:

```bash
AGENT_REACH_DISABLED_CHANNELS=reddit,linkedin agent-reach execute --task-file tasks.json
```

La comparación de nombres ignora mayúsculas y espacios; no hay coincidencias de prefijos (escribir `red` no deshabilitará `reddit`). Se aplica tanto al CLI como al servidor MCP.

### Claves de configuración reconocidas

`twitter_auth_token` · `twitter_ct0` · `groq_api_key` · `openai_api_key` · `github_token` · `reddit_client_id` · `reddit_client_secret` · `reddit_user_agent` · `linkedin_username` · `linkedin_password` · `exa_api_key` · `proxy` · `disabled_channels`

---

## 📖 4. Uso y Ejemplos

### A. Línea de comandos

Subcomandos ejecutando `agent-reach --help`:

```text
install     Set up the config directory (external tools are not installed)
configure   Set a config value or auto-extract from browser
doctor      Check platform availability
skill       Manage agent skill registration
execute     Execute tasks from a JSON file
transcribe  Transcribe a local audio/video file
```

### B. Ejecución de tareas

Los canales se manipulan usando un archivo de tareas (todos los 17 canales son soportados vía CLI):

```json
[
  {
    "id": "task-1",
    "channel": "turath",
    "action": "search",
    "args": ["الاجتهاد"]
  },
  {
    "id": "task-2",
    "channel": "mevzuat",
    "action": "mevzuat",
    "args": ["5237"]
  },
  {
    "id": "task-3",
    "channel": "uinjkt",
    "action": "journals",
    "args": []
  }
]
```

```bash
agent-reach execute --task-file tasks.json --output log.json
agent-reach execute --task-file tasks.json --continue-on-error --verbose
```

Esquema de salida (`log.json`):

```json
{
  "total_duration_ms": 1240,
  "success": true,
  "results": [
    {
      "task_id": "task-1",
      "success": true,
      "channel": "turath",
      "backend": "turath-api",
      "duration_ms": 812,
      "output": "…",
      "error": null
    }
  ]
}
```

### C. Transcripción local

```bash
agent-reach transcribe recording.mp3
agent-reach transcribe lecture.mkv --provider groq --output lecture.txt
```

Solo se aceptan archivos locales. Si ni `groq_api_key` ni `openai_api_key` están configuradas, el comando lo indica explícitamente y sugiere usar `configure`.

### D. Análisis multimedia puramente en Rust (`MediaInspector`)

```rust
use agent_reach_core::MediaInspector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meta = MediaInspector::inspect_file("sample_audio.mp3")?;

    println!("Format: {}", meta.format_name);          // symphonia-native
    println!("Codec: {}", meta.codec_name);
    println!("Sample rate: {} Hz", meta.sample_rate);
    println!("Channels: {}", meta.channels);
    println!("Duration: {:.2} seconds", meta.duration_seconds);

    Ok(())
}
```

### E. Servidor MCP

El binario `agent-reach-mcp` interactúa mediante JSON-RPC sobre stdio (protocolo `2024-11-05`) y expone cinco herramientas:

| Herramienta | Parámetros |
| :--- | :--- |
| `web_read` | `url` |
| `rss_fetch` | `url` |
| `rss_parse` | `xml` |
| `exa_search` | `query`, `num_results` (1–10) |
| `agent_reach_execute` | `channel`, `action`, `args[]` |

`agent_reach_execute` acepta estos doce canales: `bilibili`, `duckduckgo`, `github`, `linkedin`, `reddit`, `turath`, `twitter`, `v2ex`, `xiaohongshu`, `xiaoyuzhou`, `xueqiu`, `youtube`. Se accede a `web`, `rss` y `exa` a través de sus propias herramientas — por ende, un total de catorce canales son accesibles en MCP.

> **Honesty note:** Aunque los 17 canales se ejecutan mediante CLI con la orden `execute`, los canales `mevzuat` y `uinjkt` aún no se han añadido al `enum` de `agent_reach_execute` en MCP ni al comando `doctor`; están programados para la próxima entrega (release).

Configuración de Hermes / Claude Desktop:

```json
{
  "mcpServers": {
    "agent-reach": {
      "command": "/install/path/agent-reach-mcp"
    }
  }
}
```

---

## 🛡️ 5. Quality Gates and Tests

### Measured test status

```bash
cargo test --workspace
```

Ejecutado contra este repositorio el 7 September 2026, resultado real:

| Suite | Resultado |
| :--- | :--- |
| `agent_reach_channels` (lib) | 39 passed · 0 failed |
| `agent_reach_core` (lib) | 16 passed · 0 failed |
| `search_gauntlet` (integration) | 3 passed · 0 failed · 1 ignored |
| **Total** | **58 passed · 0 failed · 1 ignored** |

El único test ignorado (1 ignored) requiere estar conectado a internet; debe ejecutarlo manualmente con la opción `--ignored`.

### Referee system y 8 golden gates

`harness` es un crate aislado que evalúa las 8 gates por medio de `cargo run --manifest-path harness/Cargo.toml -- gates`:

1. **build** — Validación de compilación del workspace.
2. **clippy** — Obligatoriedad de acumular cero advertencias (`-D warnings`).
3. **unit tests** — Pasan positivamente todas las pruebas unitarias.
4. **formatting** — Cumplimiento de código de formato de `cargo fmt`.
5. **answer-key grep** — Garantiza que no se copien cadenas de respuestas del golden set en el código fuente (138 frases analizadas).
6. **referee watch** — Verifica que los archivos correspondientes a los evaluadores permanezcan íntegros.
7. **criteria wiring** — Confirma la evaluación de manera activa de los requisitos de aceptación.
8. **ci parity** — Garantiza que todos los checks de CI existan en las gates locales del harness.

Los umbrales residen en `harness/kabul.json`, **separados** del golden set (`min_recall_ratio: 0.9`, `max_zero_results: 0`). El golden set contiene **24 cases** en `crates/agent-reach-channels/tests/golden_search.json`.

**Un vacío registrado abiertamente:** los 24 cases van dirigidos a `github`. La condición del Ticket A — "un nuevo canal no se considera operativo hasta que gana al menos un caso en el golden set" — está **not yet met** para `turath`, `mevzuat` y `uinjkt`. Las mediciones de los canales se hicieron en vivo, pero aún no han entrado al golden set.

### Ética de medición (Measurement ethics)

- Según `docs/adr/0003-a-refusal-is-not-a-verdict.md`: un rechazo en la capa de transporte (HTTP 429, 202) es considerado **no evaluado (not counted as a capability miss)**; se marca como `Not measured`.
- Según `docs/adr/0004-a-200-is-not-a-payload-verdict.md`: Respuestas HTTP 200 sin datos reales (`require_payload`) son tratadas como errores.

---

## 👥 6. Contribuyentes

Los conteos abajo provienen de `git shortlog -sne --all` (7 September 2026, 77 commits in total).

| Nombre / Identidad | Rol y Contribuciones | Medición |
| :--- | :--- | :--- |
| **Ercan ER** ([@Ercaner1988](https://github.com/Ercaner1988)) | Dueño del proyecto y arquitecto principal; arquitectura Rust, diseño de los 17 canales, generaciones de fuentes, y ética de medición (ADR 0001–0004) | **74 commits** (43 + 22 + 9, tres firmas combinadas) |
| **copilot-swe-agent[bot]** ([@copilot-swe-agent[bot]](https://github.com/apps/copilot-swe-agent)) | PR #2: Correcciones de formato de `cargo fmt` y lint-and-format en CI | **2 commits** |
| **Devin AI** ([@devin-ai-integration[bot]](https://github.com/apps/devin-ai-integration)) | PR #1: Detección multiplataforma de ruta de Python, mitigaciones de inyección para LinkedIn/Reddit, CLI routing | **1 commit** |
| **Kassam** (Hermes agent) | Desarrollador de IA compañero; `MediaInspector`, canal `turath`, channel switch, suite de documentación en siete idiomas | Cometido bajo la firma de Ercan ER |
| **Mihenk** (Claude Opus 5) | Revisor de arquitectura y árbitro; ADR 0001–0004, referee lock, Tickets A · B · C, verificación de payload | Cometido bajo la firma de Ercan ER |
| **GitHub Copilot** (Editor Assistant) | Autocompletado de código en el editor; patrones repetitivos de Rust, `match` arms, andamiaje de pruebas | Sin firma separada de git |
| **ZAI GLM 5.3** (Zhipu AI) | Sugerencias de ladder de búsqueda y channel optimization (commit `139b713`) | Cometido bajo la firma de Ercan ER |

**Nota de honestidad (Honesty note):** Kassam, Mihenk, ZAI GLM 5.3 y el asistente editor GitHub Copilot no realizan commits bajo firmas separadas de git; sus contribuciones están registradas bajo la firma de Ercan ER. Los Bots de GitHub directos creando PRs (`copilot-swe-agent[bot]`: 2, `Devin AI`: 1) sí aparecen de manera individualizada con sus propias signatures.

---

## 📄 7. Licencia

Este proyecto se encuentra amparado the **MIT License** — visualizaciones el archivo `LICENSE`.
Copyright: 2026 Ercan ER.

Documentación: `CONTEXT.md` (glossary), `docs/adr/` (0001–0004 architecture decisions), `docs/YOL-HARITASI-KAYNAKLAR.md` (source generations), `docs/channels/mevzuat.md`, `docs/channels/KANAL-EKLEME.md`, `docs/integration/mcp.md`, `docs/channels/rss.md`.
