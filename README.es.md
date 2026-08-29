**🌍 [Türkçe](README.md) | [English](README.en.md) | [العربية](README.ar.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md)**

# Agent Reach RS (`agent-reach-rs`)

> **Motor de Lectura de Datos Multicanal, Web y Medios en Rust Puro para Agentes de IA**

`agent-reach-rs` es un ecosistema modular en Rust que permite a los agentes de IA (Hermes, Claude, Codex, OpenCode) leer datos de manera confiable, rápida e independiente en sitios web externos, redes sociales, bases de datos académicas y archivos multimedia.

---

## 🎯 1. Propósito y Características

- **Independencia de Binarios Externos de FFmpeg (`MediaInspector`):** Decodifica e inspecciona formatos de audio y medios (MP3, WAV, AAC, FLAC, OGG, MKV) nativamente en Rust puro a través de la biblioteca `symphonia` (v0.5) sin requerir un ejecutable binario externo `ffmpeg.exe`.
- **14 Lectores Multicanal:**
  - **Social y Web:** Twitter/X (Nitter / GraphQL), Reddit API, Bilibili, Xiaohongshu (XHS), V2EX, Xueqiu, LinkedIn, Xiaoyuzhou.
  - **Académico y Código:** Turath (Base de datos de derecho islámico y manuscritos), GitHub REST API, Feeds RSS/Atom.
  - **Motores de Búsqueda:** Búsqueda semántica Exa AI, Extractor HTML DuckDuckGo, Jina Web Reader.
- **Motor de Vectores Epistémicos 5D (`agent-reach-graph`):** Matriz ontológica, estética, epistemológica, moral y lingüística basada en Turso SQLite (0.7.2).
- **Integración con Servidor MCP:** Controlador de servidor JSON-RPC y CLI totalmente compatible con los estándares de Model Context Protocol (MCP).

---

## 🏗️ 2. Arquitectura y Módulos

```text
agent-reach-rs/
├── Cargo.toml                    # Configuración del workspace (symphonia, tokio, reqwest)
├── crates/
│   ├── agent-reach-core/        # Tipos principales, MediaInspector, manejo de errores, Config
│   ├── agent-reach-channels/    # Implementación de 14 lectores multicanal (YouTube, Turath, RSS, etc.)
│   ├── agent-reach-mcp/         # Controlador del servidor MCP JSON-RPC
│   └── agent-reach-cli/         # Binario cliente de línea de comandos (binary: agent-reach)
└── harness/                     # Arnés de prueba automatizado y puertas de validación árbitro
```

---

## 🚀 3. Instalación y Configuración

### Requisitos Previos
- **Toolchain de Rust:** Rust 1.75+ (`cargo` y `rustc` instalados).
- **Dependencias Externas:** NINGUNA (No se requiere binario externo de FFmpeg, Python o Node.js).

### Compilación
```bash
# Clonar el repositorio
git clone https://github.com/Ercaner1988/agent-reach-rs.git
cd agent-reach-rs

# Compilar el workspace
cargo build --release
```

El binario compilado se ubicará en `target/release/agent-reach.exe`.

---

## 📖 4. Uso y Ejemplos

### A. Inspección de Medios en Rust Puro (`MediaInspector` API)
```rust
use agent_reach_core::MediaInspector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inspeccionar audio nativamente sin invocar ffmpeg.exe
    let meta = MediaInspector::inspect_file("muestra_audio.mp3")?;
    
    println!("Códec: {}", meta.codec_name);
    println!("Frecuencia de muestreo: {} Hz", meta.sample_rate);
    println!("Canales: {}", meta.channels);
    println!("Duración: {:.2} segundos", meta.duration_seconds);
    
    Ok(())
}
```

### B. Uso de la Línea de Comandos (CLI)
```bash
# Ejecutar búsqueda semántica en Exa
agent-reach --channel exa search "Max Weber legal rationalization"

# Leer manuscrito de la base de datos Turath
agent-reach --channel turath read --book 124 --page 45

# Obtener feed RSS
agent-reach --channel rss fetch "https://news.ycombinator.com/rss"
```

---

## 🛡️ 5. Puertas de Calidad y Pruebas

Protegido por 6 estrictas puertas de verificación con requisito de 100% de aprobación.

```bash
# Ejecutar todas las pruebas del workspace (41/41 puertas verdes)
cargo test --workspace
```

- **`agent-reach-core`:** 10/10 pruebas superadas (incluyendo la inspección de medios en Rust puro).
- **`agent-reach-channels`:** 28/28 pruebas superadas.
- **`search_gauntlet`:** 3/3 puertas árbitro verificadas.

---

## 👥 6. Colaboradores

| Nombre / Identidad | Rol y Contribuciones | Métricas |
| :--- | :--- | :--- |
| **Ercan Er** | Arquitecto Principal y Propietario (Arquitectura Rust) | 38 commits, código base principal |
| **Mihenk** | Auditor de Código y Guardián de Puertas Árbitro | Aprobaciones de árbitro y auditoría Gauntlet |
| **El-Kassâm** | Desarrollador Agente (MediaInspector, integración Rust puro) | 12 commits, medios y suite de pruebas |
| **ZAI GLM 5.3** | Contribuciones del Modelo Agente y Edición de Código | Razonamiento del Modelo |
| **GitHub Copilot** | Autocompletado de código auxiliar | Asistente de desarrollo |
| **Hermes** | Motor de Orquestación de Agentes | Entorno de ejecución de agentes |

---

## 📄 7. Licencia

Licenciado bajo la **Licencia MIT**. Consulte `LICENSE` para obtener más detalles.
