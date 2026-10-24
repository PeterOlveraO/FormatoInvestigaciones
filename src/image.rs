//! Conversión de SVG y WEBP a PNG: pdflatex no compone ninguno de los dos.
//!
//! El filtro `images.lua` llama al propio binario con `--convert-image` y deja
//! el PNG en la caché; los logos de la portada pueden usar `convert_to_png`.

use std::ffi::OsString;
use std::io::{BufWriter, Cursor};
use std::path::Path;

use crate::error::{GenerationError, Result};
use crate::project::replace_file;

/// Entra en el nombre del PNG en la caché: subirla obliga a reconvertir todo.
pub const CONVERTER_VERSION: &str = "1";

/// Tope de píxeles del PNG: una imagen mayor solo gastaría memoria y tiempo.
pub const MAX_PIXELS: u64 = 40_000_000;

/// Los px de SVG son de 96 ppp: a escala 2 el PNG lleva 192 ppp y mide lo mismo.
const SVG_SCALE: f32 = 2.0;
/// 192 ppp en píxeles por metro, la unidad de pHYs.
const SVG_PIXELS_PER_METER: u32 = 7559;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Svg,
    Webp,
}

/// El formato sale del contenido, no de la extensión. Misma regla que
/// `formato_del_contenido` en `images.lua`.
pub fn detect(bytes: &[u8]) -> Option<Kind> {
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(Kind::Webp);
    }
    let text = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let text = &text[text.iter().position(|b| !b.is_ascii_whitespace()).unwrap_or(text.len())..];
    let has_svg = || text.windows(4).any(|w| w == b"<svg");
    (text.starts_with(b"<svg") || (text.starts_with(b"<?xml") && has_svg())).then_some(Kind::Svg)
}

/// Convierte `source` (SVG o WEBP) a PNG en `destination`. Escribe en un
/// temporal de la misma carpeta y lo renombra: un corte no deja un PNG a medias.
pub fn convert_to_png(source: &Path, destination: &Path) -> Result<()> {
    let bytes = std::fs::read(source)?;
    let (width, height, rgba, pixels_per_meter) = match detect(&bytes) {
        Some(Kind::Svg) => render_svg(&bytes)?,
        Some(Kind::Webp) => decode_webp(&bytes)?,
        None => {
            return Err(GenerationError::new(tr!(
                es: "{} no es SVG ni WEBP.",
                en: "{} is not SVG or WEBP.",
                source.display()
            )));
        }
    };
    replace_file(destination, |temporary| write_png(temporary, width, height, &rgba, pixels_per_meter))?;
    Ok(())
}

type Pixels = (u32, u32, Vec<u8>, Option<u32>);

fn check_pixels(width: u64, height: u64) -> Result<()> {
    if width.saturating_mul(height) > MAX_PIXELS {
        return Err(GenerationError::new(tr!(
            es: "la imagen mide {width}×{height} píxeles, más del tope de {MAX_PIXELS}.",
            en: "the image is {width}×{height} pixels, over the limit of {MAX_PIXELS}."
        )));
    }
    Ok(())
}

fn render_svg(bytes: &[u8]) -> Result<Pixels> {
    use resvg::{tiny_skia, usvg};
    // Sin imágenes enlazadas ni incrustadas: un SVG no lee archivos ni la red.
    let mut options = usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: Box::new(|_, _, _| None),
            resolve_string: Box::new(|_, _| None),
        },
        ..Default::default()
    };
    options.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_data(bytes, &options).map_err(|e| {
        GenerationError::new(tr!(es: "el SVG no es válido: {e}", en: "the SVG is not valid: {e}"))
    })?;
    let size = tree.size();
    let (width, height) =
        ((size.width() * SVG_SCALE).ceil() as u64, (size.height() * SVG_SCALE).ceil() as u64);
    check_pixels(width, height)?;
    let mut pixmap = tiny_skia::Pixmap::new(width as u32, height as u32).ok_or_else(|| {
        GenerationError::new(tr!(es: "el SVG no tiene tamaño.", en: "the SVG has no size."))
    })?;
    resvg::render(&tree, tiny_skia::Transform::from_scale(SVG_SCALE, SVG_SCALE), &mut pixmap.as_mut());
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let color = pixel.demultiply();
            [color.red(), color.green(), color.blue(), color.alpha()]
        })
        .collect();
    Ok((pixmap.width(), pixmap.height(), rgba, Some(SVG_PIXELS_PER_METER)))
}

// WEBP no guarda resolución: el PNG sale sin pHYs, como cualquier PNG sin ella.
fn decode_webp(bytes: &[u8]) -> Result<Pixels> {
    let invalid = |e: ::image::ImageError| {
        GenerationError::new(tr!(es: "el WEBP no es válido: {e}", en: "the WEBP is not valid: {e}"))
    };
    let reader = || ::image::ImageReader::with_format(Cursor::new(bytes), ::image::ImageFormat::WebP);
    let (width, height) = reader().into_dimensions().map_err(invalid)?;
    check_pixels(width.into(), height.into())?;
    let image = reader().decode().map_err(invalid)?.into_rgba8();
    Ok((width, height, image.into_raw(), None))
}

fn write_png(
    path: &Path,
    width: u32,
    height: u32,
    rgba: &[u8],
    pixels_per_meter: Option<u32>,
) -> std::io::Result<()> {
    let mut encoder = png::Encoder::new(BufWriter::new(std::fs::File::create(path)?), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_pixel_dims(pixels_per_meter.map(|ppm| png::PixelDimensions {
        xppu: ppm,
        yppu: ppm,
        unit: png::Unit::Meter,
    }));
    let mut writer = encoder.write_header().map_err(std::io::Error::other)?;
    writer.write_image_data(rgba).map_err(std::io::Error::other)?;
    writer.finish().map_err(std::io::Error::other)
}

/// `--convert-image <ORIGEN> <DESTINO>`: lo llama el filtro de imágenes. Sin
/// menú, ajustes ni registro; devuelve el código de salida.
pub fn run_cli(args: &[OsString]) -> i32 {
    let [source, destination] = args else {
        eprintln!(
            "{}",
            tr!(
                es: "uso: investigacion --convert-image <ORIGEN> <DESTINO>",
                en: "usage: investigacion --convert-image <SOURCE> <DESTINATION>"
            )
        );
        return 2;
    };
    exit_after(crate::process::TOOL_TIMEOUT);
    match convert_to_png(Path::new(source), Path::new(destination)) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{}: {error}", Path::new(source).display());
            1
        }
    }
}

/// Termina el proceso (código 124) si sigue vivo tras `limit`. `pandoc.pipe` no
/// tiene límite: si a Pandoc lo corta el suyo, este proceso quedaría huérfano
/// y un SVG con muchos filtros puede tardar más de cinco minutos en dibujarse.
fn exit_after(limit: std::time::Duration) {
    std::thread::spawn(move || {
        std::thread::sleep(limit);
        eprintln!(
            "{}",
            tr!(
                es: "la conversión tardó más de {} s y se detuvo.",
                en: "the conversion took more than {} s and was stopped.",
                limit.as_secs()
            )
        );
        std::process::exit(124);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    // El límite termina el proceso, así que la prueba se relanza a sí misma
    // como hijo: el hijo se queda dormido y el límite tiene que cortarlo.
    #[test]
    fn a_conversion_that_does_not_end_is_stopped() {
        if std::env::var_os("INVESTIGACION_TEST_EXIT_AFTER").is_some() {
            exit_after(std::time::Duration::from_millis(100));
            std::thread::sleep(std::time::Duration::from_secs(30));
            return;
        }
        let started = std::time::Instant::now();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "image::tests::a_conversion_that_does_not_end_is_stopped", "--nocapture"])
            .env("INVESTIGACION_TEST_EXIT_AFTER", "1")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(124));
        assert!(started.elapsed() < std::time::Duration::from_secs(10));
    }

    const SQUARE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="30" height="20"><rect width="30" height="20" fill="red"/></svg>"#;

    #[test]
    fn the_format_comes_from_the_content() {
        assert_eq!(detect(SQUARE.as_bytes()), Some(Kind::Svg));
        assert_eq!(
            detect(format!("\u{FEFF}\n  <?xml version=\"1.0\"?>\n{SQUARE}").as_bytes()),
            Some(Kind::Svg)
        );
        assert_eq!(detect(b"RIFF\x10\0\0\0WEBPVP8 "), Some(Kind::Webp));
        assert_eq!(detect(b"<?xml version=\"1.0\"?><html/>"), None);
        assert_eq!(detect(b"RIFF\x10\0\0\0WAVEfmt "), None);
        assert_eq!(detect(b"\x89PNG\r\n\x1a\n"), None);
        assert_eq!(detect(b""), None);
    }

    #[test]
    fn an_svg_becomes_a_png_at_192_dpi() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("a.svg");
        let destination = directory.path().join("a.png");
        std::fs::write(&source, SQUARE).unwrap();
        convert_to_png(&source, &destination).unwrap();
        let decoder = png::Decoder::new(BufReader::new(std::fs::File::open(&destination).unwrap()));
        let reader = decoder.read_info().unwrap();
        let info = reader.info();
        assert_eq!((info.width, info.height), (60, 40));
        assert_eq!(info.pixel_dims.map(|d| d.xppu), Some(SVG_PIXELS_PER_METER));
        // Sin temporales sueltos en la carpeta.
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 2);
    }

    #[test]
    fn an_image_over_the_pixel_limit_fails_before_rendering() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("enorme.svg");
        let destination = directory.path().join("enorme.png");
        std::fs::write(
            &source,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100000" height="100000"/>"#,
        )
        .unwrap();
        assert!(convert_to_png(&source, &destination).is_err());
        assert!(check_pixels(8000, 5000).is_ok() && check_pixels(8000, 5001).is_err());
    }

    #[test]
    fn a_failed_conversion_leaves_the_cached_png_untouched() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("roto.svg");
        let destination = directory.path().join("roto.png");
        std::fs::write(&source, "<svg <<<").unwrap();
        std::fs::write(&destination, b"anterior").unwrap();
        assert!(convert_to_png(&source, &destination).is_err());
        assert_eq!(std::fs::read(&destination).unwrap(), b"anterior");
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 2);
    }

    // Un SVG no lee archivos: las imágenes enlazadas quedan en blanco aunque
    // existan. El PNG no se vería igual (resvg va sin mapas de bits); el SVG
    // enlazado sí, y es el que hace fallar la prueba si se quita el bloqueo.
    #[test]
    fn an_svg_does_not_load_linked_local_files() {
        let directory = tempfile::tempdir().unwrap();
        let linked_svg = directory.path().join("archivo-local.svg");
        let linked_png = directory.path().join("archivo-local.png");
        std::fs::write(&linked_svg, SQUARE).unwrap();
        convert_to_png(&linked_svg, &linked_png).unwrap();
        let source = directory.path().join("enlace.svg");
        let destination = directory.path().join("enlace.png");
        // Rutas absolutas: sin el bloqueo, resvg las cargaría desde cualquier carpeta.
        let image = |path: &Path| format!(r#"<image href="{}" width="30" height="20"/>"#, path.display());
        std::fs::write(
            &source,
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="30" height="20">{}{}</svg>"#,
                image(&linked_png),
                image(&linked_svg)
            ),
        )
        .unwrap();
        convert_to_png(&source, &destination).unwrap();
        let mut reader = png::Decoder::new(BufReader::new(std::fs::File::open(&destination).unwrap()))
            .read_info()
            .unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut pixels).unwrap();
        assert!(pixels.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn a_webp_keeps_its_pixels() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/sample-photo.webp");
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("foto.png");
        convert_to_png(&source, &destination).unwrap();
        let (width, height) = ::image::ImageReader::open(&source).unwrap().into_dimensions().unwrap();
        let reader = png::Decoder::new(BufReader::new(std::fs::File::open(&destination).unwrap()))
            .read_info()
            .unwrap();
        assert_eq!((reader.info().width, reader.info().height), (width, height));
        assert!(reader.info().pixel_dims.is_none());
    }
}
