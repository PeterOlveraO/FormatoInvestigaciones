# AGENTS.md

Proyecto en Rust (CLI + menú interactivo) que convierte una investigación
escrita en Markdown al formato APA en PDF.

Reglas que no se negocian al añadir código:

- **Proyecto en inglés, comentarios en español.** Identificadores, módulos,
  archivos, carpetas, opciones y mensajes del CLI/TUI van en inglés; los
  comentarios, en español y breves (uno por bloque, y por línea solo en casos
  especiales). Lo que lee el lector del PDF (plantillas) sigue en español.
- **Dependencias mínimas.** Cada crate nuevo tiene que justificarse; Pandoc,
  pdflatex y Graphviz son dependencias del sistema, no del crate.
- **Nada específico de un sistema operativo**: el programa tiene que funcionar
  igual en Linux, Windows y macOS.
- Antes de terminar: `cargo fmt`, `cargo clippy --all-targets -- -D warnings` y
  `cargo test`.

El resto de las convenciones —arquitectura y los contratos entre las piezas—
está en `CLAUDE.md`.
