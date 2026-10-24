--[[
Funciones que comparten los filtros. Pandoc no deja que un filtro haga
`require` de un archivo vecino, así que el generador pega este texto al
principio de cada filtro al escribirlo (ver `pandoc.rs`).
]]

local utils = require("pandoc.utils")

-- Idioma de la interfaz, que pasa el generador; por omisión, español.
local INGLES = os.getenv("INVESTIGACION_LANG") == "en"
local function texto(es, en)
  if INGLES then return en end
  return es
end

-- El prefijo lo reconoce el generador para reenviar el aviso a la terminal.
-- Cada aviso sale una sola vez.
local avisados = {}
local function avisar(mensaje)
  if avisados[mensaje] then return end
  avisados[mensaje] = true
  io.stderr:write("[investigacion] " .. mensaje .. "\n")
end

-- Un archivo existe si se puede leer. En Linux `io.open` también abre las
-- carpetas, pero leerlas da error: por eso se comprueba la lectura.
local function existe(ruta)
  local archivo = io.open(ruta, "rb")
  if not archivo then return false end
  local _, error_de_lectura = archivo:read(1)
  archivo:close()
  return error_de_lectura == nil
end

-- Escribe en un temporal y lo renombra: si Pandoc se corta a medias, no queda
-- con el nombre final un archivo incompleto que `existe` daría por bueno.
local function escribir(ruta, contenido)
  local temporal = ruta .. ".tmp"
  local archivo = io.open(temporal, "wb")
  if not archivo then return false end
  local escrito = archivo:write(contenido)
  if archivo:close() and escrito then
    os.remove(ruta) -- en Windows `os.rename` no reemplaza un archivo existente
    if os.rename(temporal, ruta) then return true end
  end
  os.remove(temporal)
  return false
end

-- Si el bloque lleva alguna de las clases (`{ dot = true, ... }`), sin
-- distinguir mayúsculas.
local function tiene_clase(bloque, clases)
  for _, clase in ipairs(bloque.classes) do
    if clases[clase:lower()] then return true end
  end
  return false
end

-- El pie se escribe en Markdown, así que puede llevar formato. Se lee con el
-- mismo lector que el cuerpo (sin LaTeX crudo salvo con --allow-latex).
local FORMATO_MARKDOWN = os.getenv("INVESTIGACION_MARKDOWN_FORMAT") or "markdown"
local function pie_de_figura(markdown)
  if not markdown or markdown == "" then return nil end
  return utils.blocks_to_inlines(pandoc.read(markdown, FORMATO_MARKDOWN).blocks)
end

