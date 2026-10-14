#!/usr/bin/env sh
# Instala investigacion en Linux: revisa las herramientas, compila con cargo
# (que descarga solo las librerías de Cargo.lock) y borra lo que sobra.
# Installs investigacion on Linux: checks the tools, builds with cargo and cleans up.
set -e
cd "$(dirname "$0")"

missing=""
for tool in cargo pandoc pdflatex; do
  command -v "$tool" >/dev/null 2>&1 || missing="$missing $tool"
done
if [ -n "$missing" ]; then
  echo "Falta instalar / Missing:$missing  →  INSTALL.md"
  exit 1
fi
command -v dot >/dev/null 2>&1 ||
  echo "Aviso: sin Graphviz (dot) los diagramas salen como código. / Warning: without Graphviz, diagrams stay as code."

echo "Instalando las librerías y el programa con cargo (la primera vez necesita internet)…"
echo "Installing the libraries and the program with cargo (the first time needs internet)…"
cargo install --path . --locked

# El programa ya está en ~/.cargo/bin: la compilación intermedia solo ocupa espacio.
cargo clean --release
echo "Listo. Abre el menú con: investigacion   /   Done. Open the menu with: investigacion"
