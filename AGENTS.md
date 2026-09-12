# AGENTS.md

Proyecto CLI en Python que convierte una investigación escrita en Markdown al
formato APA en PDF.

Dos reglas que no se negocian al añadir código:

- **Solo biblioteca estándar de Python.** El paquete no tiene dependencias y así
  debe seguir; Pandoc, pdflatex y Graphviz son dependencias del sistema, no de
  Python.
- **Todo se trabaja dentro del entorno virtual** (`.venv`), en modo editable.

El resto de las convenciones —idioma, arquitectura y los contratos entre las
piezas— está en `CLAUDE.md`.
