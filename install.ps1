# Instala investigacion en Windows: revisa las herramientas, compila con cargo
# (que descarga solo las librerías de Cargo.lock) y borra lo que sobra.
# Installs investigacion on Windows: checks the tools, builds with cargo and cleans up.
# Uso / Usage: powershell -ExecutionPolicy Bypass -File install.ps1
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

$missing = 'cargo', 'pandoc', 'pdflatex' | Where-Object { -not (Get-Command $_ -ErrorAction SilentlyContinue) }
if ($missing) {
    Write-Host "Falta instalar / Missing: $($missing -join ', ')  ->  INSTALL.md"
    exit 1
}
if (-not (Get-Command dot -ErrorAction SilentlyContinue)) {
    Write-Host 'Aviso: sin Graphviz (dot) los diagramas salen como código. / Warning: without Graphviz, diagrams stay as code.'
}

Write-Host 'Instalando las librerías y el programa con cargo (la primera vez necesita internet)...'
Write-Host 'Installing the libraries and the program with cargo (the first time needs internet)...'
cargo install --path . --locked
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# El programa ya está en ~\.cargo\bin: la compilación intermedia solo ocupa espacio.
cargo clean --release
Write-Host 'Listo. Abre el menú con: investigacion   /   Done. Open the menu with: investigacion'
