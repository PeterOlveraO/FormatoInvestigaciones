//! La tubería completa: Markdown → Pandoc → plantilla → pdflatex → PDF.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::compile::{compile_pdf, copy_template_assets};
use crate::document::{DocumentData, slugify};
use crate::encoding::decode_text;
use crate::error::{GenerationError, Result};
use crate::latex::render_template;
use crate::markdown::{expand_home, read_markdown, resolve_markdown_path};
use crate::pandoc::pandoc_to_latex;
use crate::project::{DEFAULT_FORMAT, Project, absolute};

/// Opciones de la generación que no son datos de la portada.
#[derive(Debug, Default, Clone)]
pub struct GenerateOptions {
    /// Nombre de `templates/` o ruta a una plantilla; `None` es `apa`.
    pub template: Option<String>,
    /// Interpreta los comandos LaTeX escritos en el Markdown.
    pub allow_raw_latex: bool,
    /// Carpeta de logos; `None` usa la de la plantilla o `templates/logos/`.
    pub logos: Option<PathBuf>,
    /// Nombre del PDF; `None` lo deriva del nombre del Markdown.
    pub file_name: Option<String>,
    /// Markdown ya leído (el CLI lo lee antes para validarlo).
    pub markdown: Option<String>,
}

/// Nombre final del PDF. Es independiente del título: por omisión sale del
/// nombre del Markdown (`Tarea2-3.md` → `tarea2-3.pdf`), y `file_name` (con o
/// sin `.pdf`) lo cambia. Así el título de la portada puede ser largo y con
/// acentos sin que el archivo herede ese nombre.
pub fn output_file_name(markdown_path: &Path, file_name: Option<&str>) -> String {
    let base = match file_name.map(str::trim).filter(|n| !n.is_empty()) {
        Some(name) => {
            let split = name.len().saturating_sub(4);
            let has_pdf = name.get(split..).is_some_and(|end| end.eq_ignore_ascii_case(".pdf"));
            slugify(if has_pdf { &name[..split] } else { name })
        }
        None => slugify(&markdown_path.file_stem().unwrap_or_default().to_string_lossy()),
    };
    format!("{base}.pdf")
}

/// Genera el PDF y devuelve su ruta.
pub fn generate_pdf(
    project: &Project,
    markdown_path: &Path,
    output_directory: &Path,
    data: &DocumentData,
    options: &GenerateOptions,
    on_warning: &mut dyn FnMut(String),
) -> Result<PathBuf> {
    let markdown_path = resolve_markdown_path(markdown_path, &project.input_dir())?;
    let markdown = match &options.markdown {
        Some(text) => text.clone(),
        None => read_markdown(&markdown_path)?,
    };
    if markdown.trim().is_empty() {
        return Err(GenerationError::new(
            tr!(es: "El archivo Markdown está vacío.", en: "The Markdown file is empty."),
        ));
    }

    let template_file = project.find_template(options.template.as_deref())?;
    let logos = project.resolve_logos_directory(&template_file, options.logos.as_deref())?;
    let template = decode_text(&std::fs::read(&template_file)?, &template_file)?;
    // Por ahora el formato es siempre APA 7; los marcadores del contrato del
    // diseño (%%FORMAT%%, %%CLASS_OPTIONS%%) se resuelven antes que los datos.
    let format_sty = project.format_sty(DEFAULT_FORMAT);
    let template = template
        .replace("%%FORMAT%%", "\\usepackage[spanish, es-tabla]{babel}\n\\usepackage{investigacion-format}")
        .replace("%%CLASS_OPTIONS%%", "12pt, letterpaper");
    let content = pandoc_to_latex(project, &markdown_path, &markdown, options.allow_raw_latex, on_warning)?;
    let rendered = render_template(&template, &content, data)?;

    let output_directory = absolute(&expand_home(output_directory));
    std::fs::create_dir_all(&output_directory)?;
    let output_pdf = output_directory.join(output_file_name(&markdown_path, options.file_name.as_deref()));

    // El temporal se borra solo; en Windows un archivo puede seguir bloqueado
    // un instante tras cerrar pdflatex, y eso no debe convertirse en error.
    let temporary = tempfile::Builder::new().prefix("investigacion-").tempdir()?;
    let tex_path = temporary.path().join("trabajo.tex");
    std::fs::write(&tex_path, rendered)?;
    copy_template_assets(
        &template_file,
        &project.common_dir(),
        Some(&format_sty).filter(|p| p.is_file()).map(PathBuf::as_path),
        temporary.path(),
        logos.as_deref(),
    )?;
    let working_directory = markdown_path.parent().unwrap_or(Path::new("."));
    let state = latex_state_dir(project, &output_pdf, &template_file);
    compile_pdf(&tex_path, &output_pdf, working_directory, Some(&state), on_warning)?;
    let _ = temporary.close();
    Ok(output_pdf)
}

/// Carpeta de caché con el `.aux`/`.toc` de este trabajo: una por PDF de
/// salida y plantilla, porque otra plantilla carga otros paquetes.
fn latex_state_dir(project: &Project, output_pdf: &Path, template: &Path) -> PathBuf {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    output_pdf.hash(&mut hasher);
    template.hash(&mut hasher);
    let stem = output_pdf.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    project.cache_dir().join("latex").join(format!("{stem}-{:016x}", hasher.finish()))
}

/// Deja una copia del PDF en cada carpeta extra, sin repetir ni copiar sobre
/// la propia carpeta de salida.
pub fn copy_pdf_to(pdf: &Path, directories: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut seen: HashSet<PathBuf> = HashSet::new();
    if let Some(parent) = pdf.parent() {
        seen.insert(absolute(parent));
    }
    let mut copies = Vec::new();
    for directory in directories {
        let destination = absolute(&expand_home(directory));
        if !seen.insert(destination.clone()) {
            continue;
        }
        if destination.exists() && !destination.is_dir() {
            return Err(GenerationError::new(tr!(
                es: "El destino de la copia no es una carpeta: {}",
                en: "The copy destination is not a folder: {}",
                destination.display()
            )));
        }
        std::fs::create_dir_all(&destination)?;
        let copy = destination.join(pdf.file_name().unwrap_or_default());
        std::fs::copy(pdf, &copy)?;
        copies.push(copy);
    }
    Ok(copies)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_the_pdf_to_each_directory() {
        let directory = tempfile::tempdir().unwrap();
        let base = directory.path();
        let pdf = base.join("salida").join("trabajo.pdf");
        std::fs::create_dir(pdf.parent().unwrap()).unwrap();
        std::fs::write(&pdf, b"%PDF-1.5 contenido").unwrap();
        let copies = copy_pdf_to(&pdf, &[base.join("usb"), base.join("nube").join("materia")]).unwrap();
        assert_eq!(copies.len(), 2);
        for copy in copies {
            assert_eq!(copy.file_name().unwrap(), "trabajo.pdf");
            assert_eq!(std::fs::read(copy).unwrap(), b"%PDF-1.5 contenido");
        }
    }

    #[test]
    fn skips_the_output_directory_and_duplicates() {
        let directory = tempfile::tempdir().unwrap();
        let base = directory.path();
        let pdf = base.join("trabajo.pdf");
        std::fs::write(&pdf, b"pdf").unwrap();
        let copies = copy_pdf_to(&pdf, &[base.to_path_buf(), base.join("usb"), base.join("usb")]).unwrap();
        assert_eq!(copies.len(), 1);
    }

    #[test]
    fn rejects_a_destination_that_is_a_file() {
        let directory = tempfile::tempdir().unwrap();
        let pdf = directory.path().join("trabajo.pdf");
        std::fs::write(&pdf, b"pdf").unwrap();
        let busy = directory.path().join("ocupado.txt");
        std::fs::write(&busy, "x").unwrap();
        assert!(copy_pdf_to(&pdf, &[busy]).is_err());
    }

    #[test]
    fn the_file_name_is_independent_of_the_title() {
        let markdown = Path::new("input/IA/Tarea2-3.md");
        assert_eq!(output_file_name(markdown, None), "tarea2-3.pdf");
        assert_eq!(output_file_name(markdown, Some("Entrega Final.pdf")), "entrega-final.pdf");
        assert_eq!(output_file_name(markdown, Some("Reporte.PDF")), "reporte.pdf");
        assert_eq!(output_file_name(markdown, Some("  ")), "tarea2-3.pdf");
    }
}
