--[[
Tablas en documentos a dos columnas (IEEE).

Pandoc escribe cada tabla como `longtable`, que LaTeX no admite en modo de dos
columnas: la compilación se detiene. Cuando el formato es a dos columnas
(INVESTIGACION_TWOCOLUMN=1) cada tabla se reescribe como un `table` flotante
con `tabularx` al ancho de la columna, que sí funciona ahí y reparte el texto
en renglones. En cualquier otro formato este filtro no hace nada.

Va al final de la cadena de filtros: el contenido de cada celda ya pasó por
los demás (HTML en línea, imágenes…) y aquí solo se escribe en LaTeX.
]]

if os.getenv("INVESTIGACION_TWOCOLUMN") ~= "1" then
  return {}
end

-- Bloques → LaTeX, con los párrafos de una celda separados por \par.
local function latex(bloques)
  local texto = pandoc.write(pandoc.Pandoc(bloques), "latex")
  texto = texto:gsub("%s+$", ""):gsub("\n\n+", " \\par ")
  return texto
end

-- Alineación de Pandoc → columna X de tabularx.
local COLUMNAS = {
  AlignLeft = ">{\\raggedright\\arraybackslash}X",
  AlignCenter = ">{\\centering\\arraybackslash}X",
  AlignRight = ">{\\raggedleft\\arraybackslash}X",
  AlignDefault = ">{\\raggedright\\arraybackslash}X",
}

local function fila(celdas)
  local partes = {}
  for _, celda in ipairs(celdas) do
    partes[#partes + 1] = latex(celda)
  end
  return table.concat(partes, " & ") .. " \\\\"
end

function Table(tabla)
  local simple = pandoc.utils.to_simple_table(tabla)
  local columnas = {}
  for _, alineacion in ipairs(simple.aligns) do
    columnas[#columnas + 1] = COLUMNAS[tostring(alineacion)] or COLUMNAS.AlignDefault
  end

  local salida = { "\\begin{table}[!ht]", "\\centering" }
  if #simple.caption > 0 then
    salida[#salida + 1] = "\\caption{" .. latex({ pandoc.Plain(simple.caption) }) .. "}"
  end
  salida[#salida + 1] = "\\footnotesize"
  salida[#salida + 1] = "\\begin{tabularx}{\\columnwidth}{" .. table.concat(columnas) .. "}"
  salida[#salida + 1] = "\\toprule"
  local encabezado = false
  for _, celda in ipairs(simple.headers) do
    if #celda > 0 then encabezado = true end
  end
  if encabezado then
    salida[#salida + 1] = fila(simple.headers)
    salida[#salida + 1] = "\\midrule"
  end
  for _, renglon in ipairs(simple.rows) do
    salida[#salida + 1] = fila(renglon)
  end
  salida[#salida + 1] = "\\bottomrule"
  salida[#salida + 1] = "\\end{tabularx}"
  salida[#salida + 1] = "\\end{table}"
  return pandoc.RawBlock("latex", table.concat(salida, "\n"))
end
