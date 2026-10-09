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

3. Listas muy anidadas. LaTeX admite 4 niveles de viñetas, 4 de números y 6
   en total; más allá se detiene con «Too deeply nested». Lo que pase del
   límite se sube al último nivel permitido, con un aviso.
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
  ["obras citadas"] = true,
  ["fuentes consultadas"] = true,
  ["fuentes"] = true,
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

-- Avisos en el idioma de la interfaz, con el prefijo que reenvía el generador.
local function avisar(es, en)
  local mensaje = os.getenv("INVESTIGACION_LANG") == "en" and en or es
  io.stderr:write("[investigacion] " .. mensaje .. "\n")
end

local function es_lista(bloque)
  return bloque.t == "BulletList" or bloque.t == "OrderedList"
end

local recortada = false

-- `vinetas` y `numeros` cuentan las listas que ya envuelven a esta.
local function recorta(lista, vinetas, numeros)
  if lista.t == "BulletList" then vinetas = vinetas + 1 else numeros = numeros + 1 end
  local elementos = {}
  for _, elemento in ipairs(lista.content) do
    local propio, subidos = {}, {}
    for _, bloque in ipairs(elemento) do
      local cabe = es_lista(bloque) and vinetas + numeros < 6
        and (bloque.t == "BulletList" and vinetas or numeros) < 4
      if cabe then
        propio[#propio + 1] = recorta(bloque, vinetas, numeros)
      elseif es_lista(bloque) then
        -- Sus elementos pasan a ser hermanos de este, en esta misma lista.
        recortada = true
        local copia = lista:clone()
        copia.content = bloque.content
        for _, hermano in ipairs(recorta(copia, vinetas - (lista.t == "BulletList" and 1 or 0),
            numeros - (lista.t == "OrderedList" and 1 or 0)).content) do
          subidos[#subidos + 1] = hermano
        end
      else
        propio[#propio + 1] = bloque
      end
    end
    elementos[#elementos + 1] = propio
    for _, hermano in ipairs(subidos) do elementos[#elementos + 1] = hermano end
  end
  lista.content = elementos
  return lista
end

local function recorta_listas(documento)
  local raiz = function(lista) return recorta(lista, 0, 0), false end
  documento = documento:walk({ traverse = "topdown", BulletList = raiz, OrderedList = raiz })
  if recortada then
    avisar("Una lista tiene más niveles de los que LaTeX admite (4 de viñetas o de números); los más profundos se subieron al último nivel.",
      "A list is nested deeper than LaTeX allows (4 levels of bullets or numbers); the deepest levels were moved up to the last one.")
  end
  return documento
end

function Pandoc(documento)
  documento = recorta_listas(documento)
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
