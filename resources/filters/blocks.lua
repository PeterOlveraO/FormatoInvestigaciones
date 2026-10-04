--[[
Dos arreglos de presentación que no se pueden hacer solo desde la plantilla,
porque dependen de lo que el documento diga y no de cómo se compone.

1. Cajas de nota. Una división con valla (`::: nota` … `:::`) se convierte en el
   entorno CajaMarcada de la plantilla, con su título en negrita. Sin esto
   Pandoc descarta la división y el texto sale como un párrafo más.

2. Referencias con sangría francesa. APA 7 pide que la primera línea de cada
   referencia vaya al margen y las demás sangradas. Aquí se localiza el
   encabezado «Referencias», se deshace la lista de viñetas si se escribió así y
   se envuelve todo en el entorno ReferenceList, que cada formato define
   (sangría francesa en APA, Harvard y MLA).
]]

local utils = require("pandoc.utils")
local texto = require("pandoc.text")

-- Nombre de la división → título que sale en la caja, en el idioma del
-- documento (INVESTIGACION_DOC_LANG). Valen los nombres en español y en inglés.
local INGLES = os.getenv("INVESTIGACION_DOC_LANG") == "en"
local function caja(es, en) if INGLES then return en end return es end
local CAJAS = {
  nota = caja("Nota", "Note"), note = caja("Nota", "Note"),
  aviso = caja("Aviso", "Warning"), warning = caja("Aviso", "Warning"),
  importante = caja("Importante", "Important"), important = caja("Importante", "Important"),
  ejemplo = caja("Ejemplo", "Example"), example = caja("Ejemplo", "Example"),
  definicion = caja("Definición", "Definition"), ["definición"] = caja("Definición", "Definition"),
  definition = caja("Definición", "Definition"),
}

-- Encabezados que abren la lista de referencias, con y sin acento.
local REFERENCIAS = {
  ["referencias"] = true,
  ["bibliografia"] = true,
  ["bibliografía"] = true,
  ["referencias bibliograficas"] = true,
  ["referencias bibliográficas"] = true,
  ["lista de referencias"] = true,
  ["references"] = true,
  ["reference list"] = true,
  ["bibliography"] = true,
  ["works cited"] = true,
}

local function crudo(latex)
  return pandoc.RawBlock("latex", latex)
end

-- El título puede llevar formato, así que se escribe con el propio Pandoc: de
-- paso escapa los caracteres que LaTeX tomaría como comandos.
local function a_latex(cadena)
  local documento = pandoc.Pandoc({ pandoc.Plain({ pandoc.Str(cadena) }) })
  return (pandoc.write(documento, "latex"):gsub("%s+$", ""))
end

function Div(division)
  for _, clase in ipairs(division.classes) do
    local titulo = CAJAS[texto.lower(clase)]
    if titulo then
      titulo = division.attributes["title"] or titulo
      local bloques = { crudo("\\begin{CajaMarcada}{" .. a_latex(titulo) .. "}") }
      for _, bloque in ipairs(division.content) do
        bloques[#bloques + 1] = bloque
      end
      bloques[#bloques + 1] = crudo("\\end{CajaMarcada}")
      return bloques
    end
  end
  return nil
end

local function es_encabezado_de_referencias(bloque)
  if bloque.t ~= "Header" then return false end
  return REFERENCIAS[texto.lower(utils.stringify(bloque))] == true
end

-- Una referencia por párrafo: si se escribieron como lista, cada elemento pasa
-- a ser un párrafo suelto, que es lo que necesita la sangría francesa.
local function a_parrafos(bloque)
  if bloque.t ~= "BulletList" and bloque.t ~= "OrderedList" then return { bloque } end
  local parrafos = {}
  for _, elemento in ipairs(bloque.content) do
    for _, interno in ipairs(elemento) do
      if interno.t == "Plain" then
        parrafos[#parrafos + 1] = pandoc.Para(interno.content)
      else
        parrafos[#parrafos + 1] = interno
      end
    end
  end
  return parrafos
end

function Pandoc(documento)
  local salida = {}
  local dentro = false

  for _, bloque in ipairs(documento.blocks) do
    if dentro and bloque.t == "Header" then
      salida[#salida + 1] = crudo("\\end{ReferenceList}")
      dentro = false
    end

    if es_encabezado_de_referencias(bloque) then
      salida[#salida + 1] = bloque
      salida[#salida + 1] = crudo("\\begin{ReferenceList}")
      dentro = true
    elseif dentro then
      for _, parrafo in ipairs(a_parrafos(bloque)) do
        salida[#salida + 1] = parrafo
      end
    else
      salida[#salida + 1] = bloque
    end
  end

  if dentro then
    salida[#salida + 1] = crudo("\\end{ReferenceList}")
  end

  documento.blocks = salida
  return documento
end
