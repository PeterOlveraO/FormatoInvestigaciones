--[[
Deja pasar a LaTeX los dibujos y las gráficas escritos en el Markdown.

Un bloque con clase `pgfplot` (una gráfica de datos) o `tikz` (un dibujo libre)
no es código que haya que mostrar: es una figura que hay que componer. El filtro
lo convierte en LaTeX crudo, envuelto en el entorno `tikzpicture` si el bloque
no lo trae ya, y en una figura con su pie si se indicó `caption`:

    ```{.pgfplot caption="Horas por modelo"}
    \begin{axis}[ybar, xlabel={Modelo}]
      \addplot coordinates {(Cascada,120) (Ágil,85)};
    \end{axis}
    ```

Esto es LaTeX de verdad, así que aquí sí importa la sintaxis: un error en el
bloque detiene la compilación y el CLI muestra la línea. Es la diferencia con
los bloques ```dot, que se dibujan aparte y como mucho se quedan como código.
]]

local utils = require("pandoc.utils")

local CLASES = { pgfplot = true, pgfplots = true, grafica = true, tikz = true }

local function es_grafica(bloque)
  for _, clase in ipairs(bloque.classes) do
    if CLASES[clase:lower()] then return true end
  end
  return false
end

local function pie_de_figura(texto)
  if not texto or texto == "" then return nil end
  return utils.blocks_to_inlines(pandoc.read(texto, "markdown").blocks)
end

function CodeBlock(bloque)
  if not es_grafica(bloque) then return nil end

  local codigo = bloque.text
  if not codigo:find("\\begin{tikzpicture}", 1, true) then
    codigo = "\\begin{tikzpicture}\n" .. codigo .. "\n\\end{tikzpicture}"
  end

  local pie = pie_de_figura(bloque.attributes["caption"])
  if pie then
    return pandoc.Figure(
      { pandoc.RawBlock("latex", codigo) },
      { long = { pandoc.Plain(pie) } }
    )
  end
  return {
    pandoc.RawBlock("latex", "\\begin{center}"),
    pandoc.RawBlock("latex", codigo),
    pandoc.RawBlock("latex", "\\end{center}"),
  }
end
