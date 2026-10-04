--[[
Convierte en nodos de Pandoc el HTML que Markdown permite escribir a mano para
lo que su sintaxis no cubre: <br>, <mark>, <sub>, <sup>, <u>, <kbd>, <del>...

Sin este filtro el escritor de LaTeX descarta esas etiquetas sin avisar, asi que
el renglon que el documento queria cortar sale pegado y el texto marcado sale
sin marcar. Trabaja sobre el arbol ya analizado, no sobre el texto, asi que
nunca toca lo que va dentro de un bloque de codigo.
]]

local utils = require("pandoc.utils")

-- Cada etiqueta se traduce al nodo equivalente de Pandoc; el escritor de LaTeX
-- ya sabe componer todos estos (Underline -> \ul, Strikeout -> \st, el Span de
-- clase "mark" -> \hl).
local ENVOLTURAS = {
  em = pandoc.Emph,
  i = pandoc.Emph,
  strong = pandoc.Strong,
  b = pandoc.Strong,
  del = pandoc.Strikeout,
  s = pandoc.Strikeout,
  strike = pandoc.Strikeout,
  sub = pandoc.Subscript,
  sup = pandoc.Superscript,
  u = pandoc.Underline,
  ins = pandoc.Underline,
  mark = function(contenido) return pandoc.Span(contenido, pandoc.Attr("", { "mark" })) end,
  -- <kbd> y <samp> no tienen nodo propio: se componen como codigo, que es lo
  -- que significan (una tecla o la salida literal de un programa).
  kbd = function(contenido) return pandoc.Code(utils.stringify(contenido)) end,
  samp = function(contenido) return pandoc.Code(utils.stringify(contenido)) end,
  code = function(contenido) return pandoc.Code(utils.stringify(contenido)) end,
}

local function es_html(elemento)
  return elemento.t == "RawInline" and elemento.format:match("html")
end

-- Devuelve el nombre en minusculas y si la etiqueta abre o cierra.
local function etiqueta(elemento)
  if not es_html(elemento) then return nil end
  local texto = elemento.text
  local cierre = texto:match("^</%s*([%a][%w]*)%s*>$")
  if cierre then return cierre:lower(), "cierra" end
  local apertura = texto:match("^<%s*([%a][%w]*)%s*/?>$")
    or texto:match("^<%s*([%a][%w]*)%s[^>]*>$")
  if apertura then return apertura:lower(), "abre" end
  return nil
end

-- <img src="..." alt="..."> es la unica etiqueta con atributos que interesa
-- conservar: equivale a la imagen de Markdown.
local function imagen(elemento)
  if not es_html(elemento) then return nil end
  local texto = elemento.text
  if not texto:match("^<%s*[iI][mM][gG][%s>]") then return nil end
  local ruta = texto:match('src%s*=%s*"([^"]*)"') or texto:match("src%s*=%s*'([^']*)'")
  if not ruta then return nil end
  local alterno = texto:match('alt%s*=%s*"([^"]*)"') or texto:match("alt%s*=%s*'([^']*)'") or ""
  local titulo = texto:match('title%s*=%s*"([^"]*)"') or ""
  return pandoc.Image({ pandoc.Str(alterno) }, ruta, titulo)
end

--[[
Recorre la lista emparejando aperturas con cierres. Se lleva una pila porque el
HTML se anida (<strong><mark>texto</mark></strong>); lo que queda sin cerrar se
devuelve tal cual, para no perder el texto que envolvia.
]]
function Inlines(inlines)
  local pila = { { nombre = nil, contenido = pandoc.List({}) } }
  local cambio = false

  local function cima() return pila[#pila].contenido end

  for _, elemento in ipairs(inlines) do
    local nombre, tipo = etiqueta(elemento)
    local figura = imagen(elemento)
    if figura then
      cima():insert(figura)
      cambio = true
    elseif nombre == "br" then
      cima():insert(pandoc.LineBreak())
      cambio = true
    elseif nombre == "wbr" then
      cambio = true  -- sugerencia de corte: no aporta nada en el PDF
    elseif nombre and tipo == "abre" and ENVOLTURAS[nombre] then
      pila[#pila + 1] = { nombre = nombre, contenido = pandoc.List({}) }
    elseif nombre and tipo == "cierra" and ENVOLTURAS[nombre] then
      local abierto
      for indice = #pila, 2, -1 do
        if pila[indice].nombre == nombre then abierto = indice break end
      end
      if abierto then
        -- Lo que quedo abierto por dentro se vacia sin envolver.
        while #pila > abierto do
          local suelto = table.remove(pila)
          cima():extend(suelto.contenido)
        end
        local marco = table.remove(pila)
        cima():insert(ENVOLTURAS[nombre](marco.contenido))
        cambio = true
      end
    else
      cima():insert(elemento)
    end
  end

  while #pila > 1 do
    local suelto = table.remove(pila)
    cima():extend(suelto.contenido)
  end
  if not cambio then return nil end
  return pila[1].contenido
end

-- <hr> y <br> sueltos en su propio parrafo llegan como bloque.
function RawBlock(elemento)
  if not elemento.format:match("html") then return nil end
  if elemento.text:match("^<%s*[hH][rR]%s*/?>$") then return pandoc.HorizontalRule() end
  return nil
end
