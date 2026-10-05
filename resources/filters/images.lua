--[[
Resuelve las imágenes antes de que lleguen a LaTeX.

pdflatex no descarga nada: una imagen de la web (`![x](https://.../foto.png)`)
llega como \includegraphics{https://...} y la compilación falla con un «file not
found» que no explica la causa. Este filtro la baja a la caché del proyecto y
reescribe la ruta a la copia local, de modo que a partir de la segunda vez el
trabajo se genera sin internet.

De paso comprueba las imágenes locales: si la ruta no existe, en vez de romper
la compilación se avisa y queda el texto alternativo. Mismo criterio que el
resto del proyecto: avisar sin impedir que el trabajo salga.

El generador le pasa las rutas por dos variables de entorno:
  INVESTIGACION_REMOTE_IMAGES   carpeta donde se guardan las descargas
  INVESTIGACION_RESOURCES   carpetas donde buscar una imagen local, una por línea
]]

local utils = require("pandoc.utils")

local CACHE = os.getenv("INVESTIGACION_REMOTE_IMAGES")
local RECURSOS = os.getenv("INVESTIGACION_RESOURCES") or ""

-- pdflatex solo compone estos formatos. El resto (SVG, WEBP) necesitaría una
-- conversión previa, que este proyecto no hace.
local FORMATOS = { png = true, jpg = true, jpeg = true, pdf = true }
local EXTENSION_POR_TIPO = {
  ["image/png"] = "png",
  ["image/jpeg"] = "jpg",
  ["image/jpg"] = "jpg",
  ["application/pdf"] = "pdf",
}

local avisados = {}

-- Idioma de la interfaz, que pasa el generador; por omisión, español.
local INGLES = os.getenv("INVESTIGACION_LANG") == "en"
local function texto(es, en)
  if INGLES then return en end
  return es
end

-- El prefijo lo reconoce el generador para reenviar el aviso a la terminal.
local function avisar(mensaje)
  if avisados[mensaje] then return end
  avisados[mensaje] = true
  io.stderr:write("[investigacion] " .. mensaje .. "\n")
end

local function existe(ruta)
  local archivo = io.open(ruta, "rb")
  if archivo then
    archivo:close()
    return true
  end
  return false
end

local function es_remota(ruta)
  return ruta:match("^https?://") ~= nil
end

-- Extensión de la URL, sin la cadena de consulta ni el ancla.
local function extension_de(url)
  local limpia = url:gsub("[?#].*$", "")
  local extension = limpia:match("%.([%a%d]+)$")
  return extension and extension:lower() or nil
end

local function escribir(ruta, contenido)
  local archivo = io.open(ruta, "wb")
  if not archivo then return false end
  archivo:write(contenido)
  archivo:close()
  return true
end

-- Sustituye la imagen por su texto alternativo, para que el PDF se genere igual.
local function reemplazo(imagen)
  if #imagen.caption > 0 then
    return pandoc.Emph(imagen.caption)
  end
  return pandoc.Emph({ pandoc.Str("[imagen no disponible]") })
end

local function descargar(imagen)
  local extension = extension_de(imagen.src)
  local nombre = utils.sha1(imagen.src)

  -- Si ya está descargada no se vuelve a bajar: así el trabajo se genera sin red.
  for posible in pairs(FORMATOS) do
    local ruta = CACHE .. "/" .. nombre .. "." .. posible
    if existe(ruta) then return ruta end
  end

  if extension and not FORMATOS[extension] then
    avisar(texto(
      "La imagen " .. imagen.src .. " está en formato " .. extension ..
      ", que pdflatex no puede componer. Usa PNG, JPG o PDF.",
      "The image " .. imagen.src .. " is in " .. extension ..
      " format, which pdflatex cannot typeset. Use PNG, JPG or PDF."))
    return nil
  end

  local ok, tipo, contenido = pcall(pandoc.mediabag.fetch, imagen.src)
  if not ok or not contenido then
    avisar(texto(
      "No se pudo descargar la imagen " .. imagen.src .. "; revisa la dirección o tu conexión.",
      "Could not download the image " .. imagen.src .. "; check the address or your connection."))
    return nil
  end

  local final = extension or EXTENSION_POR_TIPO[(tipo or ""):gsub(";.*$", "")]
  if not final or not FORMATOS[final] then
    avisar(texto(
      "La imagen " .. imagen.src .. " no es PNG, JPG ni PDF.",
      "The image " .. imagen.src .. " is not PNG, JPG or PDF."))
    return nil
  end

  local ruta = CACHE .. "/" .. nombre .. "." .. final
  if not escribir(ruta, contenido) then
    avisar(texto("No se pudo guardar la imagen descargada en ", "Could not save the downloaded image to ") ..
      ruta .. ".")
    return nil
  end
  return ruta
end

-- Una ruta local se busca en las carpetas que pasa el generador; solo se avisa
-- cuando no aparece en ninguna.
local function encuentra_local(ruta)
  -- Tal cual: cubre tanto una ruta absoluta como una relativa al directorio de
  -- trabajo, en cualquier sistema.
  if existe(ruta) then return true end
  for carpeta in RECURSOS:gmatch("[^\n]+") do
    if existe(carpeta .. "/" .. ruta) then return true end
  end
  return false
end

-- Cuando la imagen era el único contenido de una figura, la figura se queda sin
-- nada que mostrar: se deshace para que no salga un pie de figura huérfano.
function Figure(figura)
  local tiene_imagen = false
  pandoc.walk_block(figura, {
    Image = function(imagen)
      tiene_imagen = true
      return imagen
    end,
  })
  if tiene_imagen then return nil end
  return figura.content
end

function Image(imagen)
  if es_remota(imagen.src) then
    if not CACHE then return nil end
    local ruta = descargar(imagen)
    if not ruta then return reemplazo(imagen) end
    -- Solo el nombre: pdflatex lo busca en la caché por TEXINPUTS (ver diagrams.lua).
    imagen.src = ruta:match("[^/]+$")
    return imagen
  end

  if RECURSOS ~= "" and not encuentra_local(imagen.src) then
    avisar(texto(
      "No se encontró la imagen " .. imagen.src .. "; la ruta se busca desde la carpeta del Markdown.",
      "Image not found: " .. imagen.src .. "; the path is resolved from the Markdown folder."))
    return reemplazo(imagen)
  end
  return nil
end
