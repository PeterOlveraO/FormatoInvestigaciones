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

El formato se lee del contenido. SVG y WEBP, que pdflatex no compone, se
convierten a PNG en la caché llamando al propio programa (`--convert-image`).

El generador le pasa las rutas por dos variables de entorno:
  INVESTIGACION_REMOTE_IMAGES   carpeta donde se guardan las descargas
  INVESTIGACION_RESOURCES   carpetas donde buscar una imagen local, una por línea
  INVESTIGACION_UNSEARCHABLE   carpetas de esa lista que LaTeX no puede buscar
  INVESTIGACION_CONVERTER   el programa que convierte SVG y WEBP (si falta, no se convierten)
  INVESTIGACION_CONVERTER_VERSION   entra en el nombre del PNG convertido
]]

local CACHE = os.getenv("INVESTIGACION_REMOTE_IMAGES")
local RECURSOS = os.getenv("INVESTIGACION_RESOURCES") or ""
local CONVERSOR = os.getenv("INVESTIGACION_CONVERTER")
local VERSION_CONVERSOR = os.getenv("INVESTIGACION_CONVERTER_VERSION") or ""
local INACCESIBLES = {}
for carpeta in (os.getenv("INVESTIGACION_UNSEARCHABLE") or ""):gmatch("[^\n]+") do
  INACCESIBLES[carpeta] = true
end

-- pdflatex compone PNG, JPG y PDF; SVG y WEBP se convierten antes a PNG.
local FORMATOS = { png = true, jpg = true, jpeg = true, pdf = true, svg = true, webp = true }

-- El formato se lee de los primeros bytes: ni la extensión ni el tipo que manda
-- el servidor garantizan qué es el archivo. Misma regla que `detect` en image.rs.
local function formato_del_contenido(contenido)
  if contenido:sub(1, 8) == "\137PNG\r\n\26\n" then return "png" end
  if contenido:sub(1, 3) == "\255\216\255" then return "jpg" end
  if contenido:sub(1, 4) == "%PDF" then return "pdf" end
  if contenido:sub(1, 4) == "RIFF" and contenido:sub(9, 12) == "WEBP" then return "webp" end
  -- SVG: tras un BOM y espacios opcionales, `<svg` o `<?xml` con un `<svg` después.
  local inicio = contenido:find("%S", contenido:sub(1, 3) == "\239\187\191" and 4 or 1)
  if inicio and (contenido:sub(inicio, inicio + 3) == "<svg"
      or (contenido:sub(inicio, inicio + 4) == "<?xml" and contenido:find("<svg", inicio, true))) then
    return "svg"
  end
  return nil
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

-- Sustituye la imagen por su texto alternativo, para que el PDF se genere igual.
local function reemplazo(imagen)
  if #imagen.caption > 0 then
    return pandoc.Emph(imagen.caption)
  end
  return pandoc.Emph({ pandoc.Str("[imagen no disponible]") })
end

-- Antes de descargar: avisa si la extensión es de un formato que no se puede
-- componer ni convertir (GIF…); sin extensión no se puede saber y se deja pasar.
local function formato_admitido(ruta, extension)
  if not extension or FORMATOS[extension] then return true end
  avisar(texto(
    "La imagen " .. ruta .. " está en formato " .. extension ..
    ", que pdflatex no puede componer. Usa PNG, JPG, PDF, SVG o WEBP.",
    "The image " .. ruta .. " is in " .. extension ..
    " format, which pdflatex cannot typeset. Use PNG, JPG, PDF, SVG or WEBP."))
  return false
end

-- SVG o WEBP → PNG en la caché, con nombre por contenido: no se reconvierte.
-- Devuelve el nombre para LaTeX, o nil tras avisar.
local function convertir(origen, contenido, etiqueta, formato)
  if not CONVERSOR then
    avisar(texto(
      "La imagen " .. etiqueta .. " es " .. formato:upper() .. " y no hay conversor disponible; queda el texto alternativo.",
      "The image " .. etiqueta .. " is " .. formato:upper() .. " and no converter is available; the alt text is used."))
    return nil
  end
  if not CACHE then return nil end
  local nombre = utils.sha1(VERSION_CONVERSOR .. contenido) .. ".png"
  local destino = CACHE .. "/" .. nombre
  if existe(destino) then return nombre end
  -- El motivo del fallo lo escribe el conversor en stderr (va al registro).
  local ok = pcall(pandoc.pipe, CONVERSOR, { "--convert-image", origen, destino }, "")
  if not ok or not existe(destino) then
    avisar(texto(
      "No se pudo convertir la imagen " .. etiqueta .. " (" .. formato:upper() .. ") a PNG; queda el texto alternativo.",
      "Could not convert the image " .. etiqueta .. " (" .. formato:upper() .. ") to PNG; the alt text is used."))
    return nil
  end
  return nombre
end

local function descargar(imagen)
  local extension = extension_de(imagen.src)
  local nombre = utils.sha1(imagen.src)

  -- Si ya está descargada no se vuelve a bajar: así el trabajo se genera sin red.
  -- Una descarga vieja que no es imagen (versiones anteriores la guardaban) se ignora.
  for posible in pairs(FORMATOS) do
    local ruta = CACHE .. "/" .. nombre .. "." .. posible
    local archivo = io.open(ruta, "rb")
    if archivo then
      local contenido = archivo:read("a") or ""
      archivo:close()
      if formato_del_contenido(contenido) then return ruta end
    end
  end

  if not formato_admitido(imagen.src, extension) then return nil end

  local ok, _, contenido = pcall(pandoc.mediabag.fetch, imagen.src)
  if not ok or not contenido then
    avisar(texto(
      "No se pudo descargar la imagen " .. imagen.src .. "; revisa la dirección o tu conexión.",
      "Could not download the image " .. imagen.src .. "; check the address or your connection."))
    return nil
  end

  local final = formato_del_contenido(contenido)
  if not final then
    avisar(texto(
      "La imagen " .. imagen.src .. " no es PNG, JPG, PDF, SVG ni WEBP.",
      "The image " .. imagen.src .. " is not PNG, JPG, PDF, SVG or WEBP."))
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

-- Una ruta local se busca en las carpetas que pasa el generador; devuelve
-- dónde está y de qué carpeta sale (nil si fue tal cual), o nil si no aparece.
local function encuentra_local(ruta)
  for carpeta in RECURSOS:gmatch("[^\n]+") do
    if existe(carpeta .. "/" .. ruta) then return carpeta .. "/" .. ruta, carpeta end
  end
  -- Tal cual: cubre una ruta absoluta, en cualquier sistema.
  if existe(ruta) then return ruta, nil end
  return nil
end

-- Deja lista para pdflatex la imagen que está en `encontrada`. Devuelve lo que
-- recibe LaTeX: `src` si sirve tal cual, o un nombre en la caché; nil tras avisar.
-- pdflatex elige el lector por la extensión, así que si no coincide con el
-- contenido (`foto.png` que es JPG) se copia con la buena. También se copia con
-- `copiar`: pdflatex corre en el temporal y no busca por TEXINPUTS una ruta con
-- `./` o `../`, ni en una carpeta con el separador de rutas.
local function preparar(encontrada, src, etiqueta, copiar)
  local archivo = io.open(encontrada, "rb")
  local contenido = archivo and archivo:read("a") or ""
  if archivo then archivo:close() end
  local formato = formato_del_contenido(contenido)
  if not formato then
    avisar(texto(
      "La imagen " .. etiqueta .. " no es PNG, JPG, PDF, SVG ni WEBP; queda el texto alternativo.",
      "The image " .. etiqueta .. " is not PNG, JPG, PDF, SVG or WEBP; the alt text is used."))
    return nil
  end
  if formato == "svg" or formato == "webp" then return convertir(encontrada, contenido, etiqueta, formato) end
  local extension = extension_de(src)
  if extension == "jpeg" then extension = "jpg" end
  if extension == formato and not copiar then return src end
  local nombre = utils.sha1(encontrada) .. "." .. formato
  if not CACHE or not escribir(CACHE .. "/" .. nombre, contenido) then
    avisar(texto("No se pudo copiar la imagen " .. etiqueta .. " a la caché.",
      "Could not copy the image " .. etiqueta .. " to the cache."))
    return nil
  end
  return nombre
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
    local nombre = preparar(ruta, ruta:match("[^/]+$"), imagen.src, false)
    if not nombre then return reemplazo(imagen) end
    imagen.src = nombre
    return imagen
  end

  if RECURSOS == "" then return nil end
  local encontrada, carpeta = encuentra_local(imagen.src)
  -- Muchos editores escriben los espacios como %20 (`mi%20foto.png`).
  local decodificada = imagen.src:gsub("%%(%x%x)", function(hex) return string.char(tonumber(hex, 16)) end)
  if not encontrada and decodificada ~= imagen.src then
    encontrada, carpeta = encuentra_local(decodificada)
    if encontrada then imagen.src = decodificada end
  end
  if not encontrada then
    avisar(texto(
      "No se encontró la imagen " .. imagen.src .. "; la ruta se busca desde la carpeta del Markdown.",
      "Image not found: " .. imagen.src .. "; the path is resolved from the Markdown folder."))
    return reemplazo(imagen)
  end
  local copiar = imagen.src:match("^%.") ~= nil or (carpeta ~= nil and INACCESIBLES[carpeta] == true)
  local nombre = preparar(encontrada, imagen.src, imagen.src, copiar)
  if not nombre then return reemplazo(imagen) end
  if nombre == imagen.src then return nil end
  imagen.src = nombre
  return imagen
end
