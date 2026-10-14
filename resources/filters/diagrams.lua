--[[
Convierte los diagramas escritos como código en imágenes de verdad.

Un bloque con clase `dot` (o `graphviz`) se compila con Graphviz a un PDF
vectorial y se sustituye por la imagen, así que el diagrama sale con líneas y
tipografía reales en vez de arte ASCII:

    ```{.dot caption="Árbol binario de búsqueda"}
    digraph { 50 -> 30; 50 -> 70; }
    ```

El resultado se guarda en la caché del proyecto con el nombre del `sha1` del
código, de modo que un diagrama que no cambió no se vuelve a compilar.

Si Graphviz no está instalado el bloque se queda como código y se avisa una
vez: el trabajo se genera igual. La carpeta de la caché llega en la variable
INVESTIGACION_DIAGRAMS.
]]

local CACHE = os.getenv("INVESTIGACION_DIAGRAMS")
local CLASES = { dot = true, graphviz = true }

function CodeBlock(bloque)
  if not tiene_clase(bloque, CLASES) or not CACHE then return nil end

  local nombre = utils.sha1(bloque.text) .. ".pdf"
  local ruta = CACHE .. "/" .. nombre
  if not existe(ruta) then
    local ok, salida = pcall(pandoc.pipe, "dot", { "-Tpdf" }, bloque.text)
    if not ok then
      avisar(texto(
        "No se pudo dibujar un diagrama: falta Graphviz (ver INSTALL.md) o el diagrama " ..
        "tiene un error de sintaxis. Mientras tanto sale como bloque de código.",
        "Could not draw a diagram: Graphviz is missing (see INSTALL.md) or the diagram " ..
        "has a syntax error. Meanwhile it shows as a code block."))
      return nil
    end
    if not escribir(ruta, salida) then
      avisar(texto("No se pudo guardar el diagrama en ", "Could not save the diagram to ") .. ruta .. ".")
      return nil
    end
  end

  -- El tamaño se controla igual que en una imagen normal: {width=60%}
  local atributos = {}
  for _, nombre in ipairs({ "width", "height" }) do
    if bloque.attributes[nombre] then atributos[nombre] = bloque.attributes[nombre] end
  end

  local pie = pie_de_figura(bloque.attributes["caption"])
  -- A LaTeX solo llega el nombre; pdflatex lo busca en la caché por TEXINPUTS.
  -- Una ruta completa se rompe con acentos (en Windows os.getenv no da UTF-8),
  -- espacios o el `~` de las rutas cortas.
  local imagen = pandoc.Image(pie or {}, nombre, "", pandoc.Attr("", {}, atributos))
  if pie then
    return pandoc.Figure({ pandoc.Plain({ imagen }) }, { long = { pandoc.Plain(pie) } })
  end
  -- Sin pie no hay figura, pero el diagrama se centra igual; el escritor de
  -- LaTeX no hace caso de la clase «center», así que se pone a mano.
  return {
    pandoc.RawBlock("latex", "\\begin{center}"),
    pandoc.Para({ imagen }),
    pandoc.RawBlock("latex", "\\end{center}"),
  }
end
