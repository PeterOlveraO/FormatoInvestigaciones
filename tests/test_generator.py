import shutil
import unittest
from datetime import date
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from investigacion.tui import build_argv, run as tui_run
from investigacion.generator import (
    DocumentData,
    GenerationError,
    decode_latex_log,
    filter_warnings,
    format_delivery_date,
    has_disguised_spaces,
    latex_escape,
    markdown_headings,
    normalize_heading,
    normalize_markdown,
    parse_integrantes,
    render_template,
    slugify,
    summarize_latex_errors,
    unsupported_character_warnings,
    validate_markdown,
)


ROOT = Path(__file__).parents[1]


class _FullTemplate:
    """La plantilla APA junto con el preambulo comun que carga."""

    def read_text(self, encoding="utf-8"):
        parts = [ROOT / "templates" / "common" / "investigacion.sty",
                 ROOT / "templates" / "common" / "investigacion-final.sty",
                 ROOT / "templates" / "apa" / "template.ltx"]
        return "".join(part.read_text(encoding=encoding) for part in parts)


TEMPLATE_PATH = _FullTemplate()


class GeneratorTests(unittest.TestCase):
    def test_latex_escape_protects_special_characters(self):
        self.assertEqual(latex_escape(r"A&B_50%"), r"A\&B\_50\%")

    def test_format_delivery_date_is_in_spanish(self):
        self.assertEqual(format_delivery_date(date(2026, 8, 23)), "Agosto 23, 2026")

    def test_slugify_removes_accents(self):
        self.assertEqual(slugify("Álgebra: teoría y práctica"), "algebra-teoria-y-practica")

    def test_headings_are_normalized(self):
        self.assertEqual(normalize_heading("  2. Introducción  ###"), "introduccion")

    def test_validate_markdown_warns_when_sections_are_missing(self):
        warnings = validate_markdown("# Introducción\n\nTexto")
        self.assertEqual(len(warnings), 3)

    def test_read_markdown_accepts_windows_encoding(self):
        from investigacion.generator import read_markdown

        with TemporaryDirectory() as directory:
            path = Path(directory) / "trabajo.md"
            path.write_bytes("# Introducción\n\nInformación académica".encode("cp1252"))
            self.assertIn("Información", read_markdown(path))

    def test_render_template_replaces_all_markers(self):
        template = "%%UNIVERSIDAD%% %%CONTENIDO_MARKDOWN%% %%FECHA_ENTREGA%%"
        data = DocumentData("U", "F", "A", "S", "T", "M", "D", "Agosto 23, 2026")
        self.assertEqual(render_template(template, r"\section{Texto}", data), "U \\section{Texto} Agosto 23, 2026")

    def test_base_template_replaces_all_markers(self):
        template_path = TEMPLATE_PATH
        template = template_path.read_text(encoding="utf-8")
        data = DocumentData("U", "F", "A", "S", "T", "M", "D", "Agosto 23, 2026")
        rendered = render_template(template, r"\section{Texto}", data)
        self.assertNotIn("%%UNIVERSIDAD%%", rendered)
        self.assertNotIn("%%CONTENIDO_MARKDOWN%%", rendered)

    def test_render_template_detects_unknown_markers(self):
        data = DocumentData("U", "F", "A", "S", "T", "M", "D", "Agosto 23, 2026")
        with self.assertRaises(GenerationError):
            render_template("%%DOCENT%% %%CONTENIDO_MARKDOWN%%", "x", data)

    def test_render_template_requires_content_marker(self):
        data = DocumentData("U", "F", "A", "S", "T", "M", "D", "Agosto 23, 2026")
        with self.assertRaises(GenerationError):
            render_template("%%TITULO%% sin contenido", "x", data)

    def test_content_may_mention_a_marker(self):
        data = DocumentData("U", "F", "A", "S", "T", "M", "D", "Agosto 23, 2026")
        rendered = render_template("%%CONTENIDO_MARKDOWN%%", "habla de %%TITULO%%", data)
        self.assertEqual(rendered, "habla de %%TITULO%%")

    def test_headings_inside_code_blocks_are_ignored(self):
        markdown = "# Introducción\n\n```python\n# Desarrollo\n```\n\n# Conclusión\n"
        self.assertEqual(markdown_headings(markdown), ["introduccion", "conclusion"])

    def test_validate_markdown_reports_wrong_order(self):
        markdown = "# Desarrollo\n\n# Introducción\n\n# Conclusión\n\n# Referencias\n"
        warnings = validate_markdown(markdown)
        self.assertTrue(any("orden" in warning for warning in warnings))

    def test_validate_markdown_accepts_the_recommended_structure(self):
        markdown = "# Introducción\n\n# Desarrollo\n\n# Conclusión\n\n# Referencias\n"
        self.assertEqual(validate_markdown(markdown), [])

    def test_summarize_latex_errors_keeps_only_the_real_error(self):
        log = (
            "(/usr/share/texmf-dist/tex/latex/base/article.cls)\n"
            "(/usr/share/texmf-dist/tex/latex/hyperref/hyperref.sty)\n"
            "/tmp/x/trabajo.tex:228: LaTeX Error: No counter 'none' defined.\n"
            "\n"
            "See the LaTeX manual for explanation.\n"
            "l.228 ...width - 6\\tabcolsep) * \\real{0.2500}}@{}}\n"
        )
        summary = summarize_latex_errors(log)
        self.assertIn("No counter 'none' defined", summary)
        self.assertNotIn("hyperref.sty", summary)


class IntegrantesTests(unittest.TestCase):
    """Los integrantes llegan en un solo argumento y salen uno por renglon."""

    def test_parse_integrantes_accepts_mixed_separators(self):
        self.assertEqual(
            parse_integrantes("Ana Ruiz, Luis Paz; Sofia Vela"),
            ("Ana Ruiz", "Luis Paz", "Sofia Vela"),
        )

    def test_parse_integrantes_trims_spaces_and_drops_empty_names(self):
        self.assertEqual(parse_integrantes("  Ana ,, Luis ,"), ("Ana", "Luis"))

    def test_parse_integrantes_of_an_empty_string_is_empty(self):
        self.assertEqual(parse_integrantes(""), ())
        self.assertEqual(parse_integrantes("   ,  ;  "), ())

    def test_render_template_joins_the_names_with_a_latex_newline(self):
        data = DocumentData(
            "U", "F", "A", "S", "T", "M", "D", "Agosto 23, 2026",
            integrantes=("Ana", "Luis"),
        )
        rendered = render_template("%%INTEGRANTES%% %%CONTENIDO_MARKDOWN%%", "x", data)
        self.assertEqual(rendered, r"Ana\\Luis x")

    def test_render_template_escapes_each_name_but_not_the_separator(self):
        data = DocumentData(
            "U", "F", "A", "S", "T", "M", "D", "Agosto 23, 2026",
            integrantes=("A&B", "C_D", "50%"),
        )
        rendered = render_template("%%INTEGRANTES%%%%CONTENIDO_MARKDOWN%%", "", data)
        self.assertEqual(rendered, r"A\&B\\C\_D\\50\%")
        self.assertNotIn(r"\textbackslash", rendered)

    def test_render_template_without_alumno_or_integrantes(self):
        data = DocumentData("U", "F", "", "S", "T", "M", "D", "Agosto 23, 2026")
        rendered = render_template("[%%ALUMNO%%][%%INTEGRANTES%%]%%CONTENIDO_MARKDOWN%%", "x", data)
        self.assertEqual(rendered, "[][]x")

    def test_base_template_compiles_the_markers_without_alumno(self):
        template_path = TEMPLATE_PATH
        template = template_path.read_text(encoding="utf-8")
        data = DocumentData("U", "F", "", "S", "T", "M", "D", "Agosto 23, 2026")
        rendered = render_template(template, r"\section{Texto}", data)
        self.assertIn(r"\newcommand{\ListaIntegrantes}{}", rendered)


class NormalizeMarkdownTests(unittest.TestCase):
    """El espacio invisible no debe impedir que Pandoc vea la estructura."""

    NBSP = " "

    def test_a_line_with_only_a_hard_space_becomes_empty(self):
        markdown = f"Parrafo uno.{self.NBSP}\n{self.NBSP} \nParrafo dos.\n"
        self.assertEqual(normalize_markdown(markdown), "Parrafo uno.\n\nParrafo dos.\n")

    def test_trailing_spaces_are_trimmed_in_a_dirty_document(self):
        markdown = f"Texto.  \n{self.NBSP}\nOtro parrafo.\n"
        self.assertEqual(normalize_markdown(markdown), "Texto.\n\nOtro parrafo.\n")

    def test_a_clean_document_keeps_its_line_break_of_two_spaces(self):
        """Dos espacios al final son sintaxis de Markdown: un salto de linea."""

        self.assertEqual(normalize_markdown("Texto.  \n"), "Texto.  \n")

    def test_a_hard_space_at_the_end_is_always_trimmed(self):
        self.assertEqual(normalize_markdown(f"Texto.{self.NBSP}\n"), "Texto.\n")

    def test_only_a_disguised_space_marks_the_document_as_dirty(self):
        self.assertFalse(has_disguised_spaces("Texto.  \nOtro.\t\n"))
        self.assertTrue(has_disguised_spaces(f"Texto.{self.NBSP}\n"))
        self.assertTrue(has_disguised_spaces(f"Parrafo.\n{self.NBSP}\nOtro.\n"))

    def test_a_marker_followed_by_a_hard_space_is_repaired(self):
        markdown = f"###{self.NBSP}1.7.3. Cotas\n"
        self.assertEqual(normalize_markdown(markdown), "### 1.7.3. Cotas\n")

    def test_repairs_bullets_quotes_and_numbered_lists(self):
        markdown = f"-{self.NBSP}item\n>{self.NBSP}cita\n1.{self.NBSP}uno\n"
        self.assertEqual(normalize_markdown(markdown), "- item\n> cita\n1. uno\n")

    def test_a_hard_space_inside_the_text_is_preserved(self):
        markdown = f"Fig.{self.NBSP}1 es la referencia\n"
        self.assertEqual(normalize_markdown(markdown), markdown)

    def test_code_blocks_are_left_untouched(self):
        markdown = "```python\ncode  \n```\n"
        self.assertEqual(normalize_markdown(markdown), markdown)

    def test_headings_are_detected_after_normalizing(self):
        markdown = (
            f"#{self.NBSP}Introducción{self.NBSP}\n{self.NBSP}\n"
            f"Texto.  \n{self.NBSP}\n#{self.NBSP}Conclusión\n"
        )
        self.assertEqual(
            markdown_headings(normalize_markdown(markdown)), ["introduccion", "conclusion"]
        )

    def test_read_markdown_normalizes_the_file(self):
        from investigacion.generator import read_markdown

        with TemporaryDirectory() as directory:
            path = Path(directory) / "trabajo.md"
            path.write_text(f"# Título  \n{self.NBSP}\nTexto.\n", encoding="utf-8")
            self.assertEqual(read_markdown(path), "# Título\n\nTexto.\n")


class UnicodeWarningTests(unittest.TestCase):
    """Los simbolos que la plantilla no sabe componer se avisan, no abortan."""

    def test_extracts_the_character_and_its_codepoint(self):
        log = (
            "LaTeX Warning: Caracter Unicode sin definir: ∮ (U+222E) on input line 27.\n"
        )
        warnings = unsupported_character_warnings(log)
        self.assertEqual(len(warnings), 1)
        self.assertIn("∮", warnings[0])
        self.assertIn("U+222E", warnings[0])

    def test_repeated_characters_are_reported_once(self):
        line = "LaTeX Warning: Caracter Unicode sin definir: ⚛ (U+269B) on input line 9.\n"
        self.assertEqual(len(unsupported_character_warnings(line * 3)), 1)

    def test_a_clean_log_produces_no_warnings(self):
        self.assertEqual(unsupported_character_warnings("Output written on trabajo.pdf"), [])

    def test_base_template_declares_the_symbols_and_the_fallback(self):
        template = (TEMPLATE_PATH).read_text(encoding="utf-8")
        self.assertIn(r"\newunicodechar{≠}{\ensuremath{\neq}}".replace("\\u2260", "≠"), template)
        self.assertIn(r"\def\UTFviii@undefined@err", template)

    def test_base_template_covers_the_ascii_art_and_the_emoji(self):
        """El arte ASCII y los emoji son el fallo de caracteres mas comun."""

        template = (TEMPLATE_PATH).read_text(encoding="utf-8")
        self.assertIn("pmboxdraw", template)      # lineas de caja (└ ─ ┼)
        self.assertIn("twemojis", template)       # emoji como imagen
        self.assertIn("►", template)          # punta de flecha de los diagramas
        self.assertIn(r"\texttwemoji", template)

    def test_base_template_replaces_the_fragile_commands_of_soul(self):
        """soul cuelga a pdflatex dentro de una tabla; los toma de ulem."""

        template = (TEMPLATE_PATH).read_text(encoding="utf-8")
        self.assertIn(r"\DeclareRobustCommand{\st}[1]{\sout{#1}}", template)
        self.assertIn(r"\DeclareRobustCommand{\ul}[1]{\uline{#1}}", template)
        self.assertIn(r"\IfFileExists{ulem.sty}{\RequirePackage[normalem]{ulem}}", template)
        self.assertIn(r"\AtBeginEnvironment{longtable}{\let\hl\ResaltadoEnTabla}", template)

    def test_base_template_defines_what_the_filters_produce(self):
        """Los filtros emiten estos entornos; sin ellos la compilacion falla."""

        template = (TEMPLATE_PATH).read_text(encoding="utf-8")
        self.assertIn(r"\newenvironment{CajaMarcada}[1]", template)
        self.assertIn(r"\newenvironment{ReferenciasAPA}", template)
        self.assertIn(r"\RequirePackage{pgfplots}", template)
        # Las barras tienen su propia paleta: sin esto salen de colores.
        self.assertIn(r"/pgfplots/bar cycle list/.style", template)
        self.assertIn(r"\floatsetup[figure]{capposition=top}", template)
        # floatrow deja las tablas pegadas al margen si no se recentran.
        self.assertIn(r"\setlength\LTleft{\fill}", template)
        # Las figuras se quedan donde se escribieron en vez de flotar.
        self.assertIn(r"\@ifundefined{floatsetup}{\def\fps@figure{htbp}}", template)

    def test_base_template_keeps_code_blocks_single_spaced(self):
        """Con el interlineado doble del documento los diagramas salen estirados."""

        template = (TEMPLATE_PATH).read_text(encoding="utf-8")
        self.assertIn(r"\newenvironment{Shaded}{\singlespacing\small}{}", template)
        self.assertIn(r"\AtBeginEnvironment{verbatim}{\singlespacing\small}", template)


class FilterWarningTests(unittest.TestCase):
    """Los avisos de los filtros Lua llegan a la terminal; el resto no."""

    def test_only_the_marked_lines_are_warnings(self):
        stderr = (
            "[WARNING] Deprecated syntax\n"
            "[investigacion] No se pudo descargar la imagen x.png\n"
            "otra linea\n"
        )
        self.assertEqual(
            filter_warnings(stderr), ["No se pudo descargar la imagen x.png"]
        )

    def test_a_clean_output_has_no_warnings(self):
        self.assertEqual(filter_warnings(""), [])


class LatexLogTests(unittest.TestCase):
    """El log de pdflatex mezcla UTF-8 con la codificacion interna de la fuente."""

    def test_each_line_is_decoded_on_its_own(self):
        raw = (
            "LaTeX Warning: Caracter Unicode sin definir: └ (U+2514) on input line 7.\n".encode("utf-8")
            + b"Underfull \\hbox in paragraph at lines 5--5: ingenier\xeda\n"
            + "Otro aviso con acento: compilación\n".encode("utf-8")
        )
        log = decode_latex_log(raw)
        self.assertIn("└ (U+2514)", log)          # el aviso no se convierte en jeroglificos
        self.assertIn("compilación", log)
        self.assertIn("Underfull", log)          # la linea en T1 no rompe la lectura

    def test_the_warnings_survive_a_log_with_font_encoded_lines(self):
        raw = (
            b"Underfull \\hbox: c\xf3digo\n"
            + "LaTeX Warning: Caracter Unicode sin definir: ⭐ (U+2B50) on input line 3.\n".encode("utf-8")
        )
        warnings = unsupported_character_warnings(decode_latex_log(raw))
        self.assertEqual(len(warnings), 1)
        self.assertIn("⭐", warnings[0])


class GrupoTests(unittest.TestCase):
    """El grupo de la materia es opcional y se omite cuando va vacio."""

    def test_render_template_replaces_the_grupo_marker(self):
        data = DocumentData(
            "U", "F", "A", "S", "T", "M", "D", "Agosto 23, 2026", grupo="7-A",
        )
        rendered = render_template("[%%GRUPO%%]%%CONTENIDO_MARKDOWN%%", "x", data)
        self.assertEqual(rendered, "[7-A]x")

    def test_render_template_escapes_the_grupo(self):
        data = DocumentData(
            "U", "F", "A", "S", "T", "M", "D", "Agosto 23, 2026", grupo="A&B_1",
        )
        rendered = render_template("%%GRUPO%%%%CONTENIDO_MARKDOWN%%", "", data)
        self.assertEqual(rendered, r"A\&B\_1")

    def test_grupo_defaults_to_empty_and_leaves_no_pending_marker(self):
        data = DocumentData("U", "F", "A", "S", "T", "M", "D", "Agosto 23, 2026")
        self.assertEqual(data.grupo, "")
        rendered = render_template("[%%GRUPO%%]%%CONTENIDO_MARKDOWN%%", "x", data)
        self.assertEqual(rendered, "[]x")

    def test_base_template_defines_the_grupo_command(self):
        template_path = TEMPLATE_PATH
        template = template_path.read_text(encoding="utf-8")
        data = DocumentData(
            "U", "F", "A", "S", "T", "M", "D", "Agosto 23, 2026", grupo="7-A",
        )
        rendered = render_template(template, r"\section{Texto}", data)
        self.assertIn(r"\newcommand{\GrupoMateria}{7-A}", rendered)


class PortadaTests(unittest.TestCase):
    """El orden de la portada es fijo, venga como venga el comando."""

    TEMPLATE = (TEMPLATE_PATH).read_text(encoding="utf-8")

    def test_the_order_of_the_cover_is_alumno_materia_docente_semestre_grupo(self):
        posiciones = [
            self.TEMPLATE.index(etiqueta)
            for etiqueta in (
                r"\textbf{ALUMNO:}",
                r"\textbf{MATERIA:}",
                r"\textbf{DOCENTE:}",
                r"\textbf{SEMESTRE:}",
                r"\textbf{GRUPO:}",
            )
        ]
        self.assertEqual(posiciones, sorted(posiciones))

    def test_the_team_replaces_the_student_in_the_same_place(self):
        # INTEGRANTES ocupa el lugar de ALUMNO, antes que MATERIA.
        self.assertLess(
            self.TEMPLATE.index(r"\textbf{INTEGRANTES:}"),
            self.TEMPLATE.index(r"\textbf{MATERIA:}"),
        )

    def test_the_cover_has_no_academic_work_label(self):
        self.assertNotIn("TRABAJO ACADÉMICO", self.TEMPLATE)


class EntradaTests(unittest.TestCase):
    """El Markdown se busca en input/ para no escribir la carpeta cada vez."""

    def crear_proyecto(self, directory: str, *rutas: str) -> Path:
        raiz = Path(directory)
        for ruta in rutas:
            archivo = raiz / "input" / ruta
            archivo.parent.mkdir(parents=True, exist_ok=True)
            archivo.write_text("# Introducción\n", encoding="utf-8")
        return raiz

    def test_a_file_is_found_inside_input(self):
        from investigacion.generator import resolve_markdown_path

        with TemporaryDirectory() as directory:
            raiz = self.crear_proyecto(directory, "Actividad1.md")
            with patch("investigacion.generator.project_root", return_value=raiz):
                encontrado = resolve_markdown_path(Path("Actividad1.md"))
            self.assertEqual(encontrado, (raiz / "input" / "Actividad1.md").resolve())

    def test_a_file_is_found_inside_a_subfolder_of_input(self):
        from investigacion.generator import resolve_markdown_path

        with TemporaryDirectory() as directory:
            raiz = self.crear_proyecto(directory, "IS/Actividad1.md")
            with patch("investigacion.generator.project_root", return_value=raiz):
                # Con la subcarpeta escrita y sin ella: las dos formas valen.
                por_ruta = resolve_markdown_path(Path("IS/Actividad1.md"))
                por_nombre = resolve_markdown_path(Path("Actividad1.md"))
            esperado = (raiz / "input" / "IS" / "Actividad1.md").resolve()
            self.assertEqual(por_ruta, esperado)
            self.assertEqual(por_nombre, esperado)

    def test_two_files_with_the_same_name_are_an_error(self):
        from investigacion.generator import resolve_markdown_path

        with TemporaryDirectory() as directory:
            raiz = self.crear_proyecto(directory, "IS/Tarea.md", "IA/Tarea.md")
            with patch("investigacion.generator.project_root", return_value=raiz):
                with self.assertRaises(GenerationError) as error:
                    resolve_markdown_path(Path("Tarea.md"))
        # El mensaje tiene que decir cuales son, para poder elegir.
        self.assertIn("IA/Tarea.md", str(error.exception))
        self.assertIn("IS/Tarea.md", str(error.exception))

    def test_an_existing_path_wins_over_the_input_folder(self):
        from investigacion.generator import resolve_markdown_path

        with TemporaryDirectory() as directory:
            raiz = self.crear_proyecto(directory, "trabajo.md")
            suelto = raiz / "trabajo.md"
            suelto.write_text("# Introducción\n", encoding="utf-8")
            with patch("investigacion.generator.project_root", return_value=raiz):
                self.assertEqual(resolve_markdown_path(suelto), suelto.resolve())


class DocenteTests(unittest.TestCase):
    """El docente es opcional: solo el titulo y la materia hacen falta."""

    def test_the_parser_does_not_require_it(self):
        from investigacion.cli import build_parser

        args = build_parser().parse_args(["t.md", "--titulo", "T", "--materia", "M"])
        self.assertIsNone(args.docente)

    def test_the_cover_omits_the_line_when_it_is_empty(self):
        template = (TEMPLATE_PATH).read_text(encoding="utf-8")
        self.assertIn(r"\ifdefempty{\NombreMaestro}{}{\textbf{DOCENTE:}", template)

    def test_render_template_accepts_an_empty_docente(self):
        data = DocumentData("U", "F", "A", "S", "T", "M", "", "Agosto 23, 2026")
        rendered = render_template("[%%DOCENTE%%]%%CONTENIDO_MARKDOWN%%", "x", data)
        self.assertEqual(rendered, "[]x")


class LogoTests(unittest.TestCase):
    """Los logos son opcionales y su carpeta se puede cambiar."""

    def test_the_template_looks_for_the_generic_names(self):
        template = (TEMPLATE_PATH).read_text(encoding="utf-8")
        for nombre in ("logo-universidad.png", "logo-facultad.png"):
            self.assertIn(r"\IfFileExists{" + nombre + "}", template)

    def test_an_explicit_directory_wins(self):
        from investigacion.generator import resolve_logos_directory

        with TemporaryDirectory() as directory:
            propia = Path(directory) / "mis-logos"
            propia.mkdir()
            plantilla = Path(directory) / "base.ltx"
            plantilla.write_text("x", encoding="utf-8")
            self.assertEqual(resolve_logos_directory(plantilla, propia), propia)

    def test_a_missing_directory_is_an_error(self):
        from investigacion.generator import resolve_logos_directory

        with TemporaryDirectory() as directory:
            plantilla = Path(directory) / "base.ltx"
            plantilla.write_text("x", encoding="utf-8")
            with self.assertRaises(GenerationError):
                resolve_logos_directory(plantilla, Path(directory) / "no-existe")

    def test_without_logos_there_is_nothing_to_copy(self):
        """Una plantilla sin carpeta de logos compila igual, sin logos."""

        from investigacion.generator import copy_template_assets, resolve_logos_directory

        with TemporaryDirectory() as directory:
            plantilla = Path(directory) / "base.ltx"
            plantilla.write_text("x", encoding="utf-8")
            destino = Path(directory) / "temporal"
            destino.mkdir()
            self.assertIsNone(resolve_logos_directory(plantilla))
            copy_template_assets(plantilla, destino, None)
            self.assertEqual(list(destino.iterdir()), [])


class ArgumentTests(unittest.TestCase):
    """El orden de las opciones no debe alterar el resultado."""

    ORDERS = (
        ["t.md", "--titulo", "T", "--materia", "M", "--docente", "D"],
        ["--titulo", "T", "--materia", "M", "--docente", "D", "t.md"],
        ["--docente", "D", "t.md", "--titulo", "T", "--materia", "M"],
        ["--materia", "M", "--titulo", "T", "t.md", "--docente", "D"],
        ["--copia", "otro", "--titulo", "T", "t.md", "--docente", "D", "--materia", "M"],
    )

    def test_option_order_does_not_matter(self):
        from investigacion.cli import build_parser

        parser = build_parser()
        for argv in self.ORDERS:
            with self.subTest(argv=argv):
                args = parser.parse_args(argv)
                self.assertEqual(args.markdown, Path("t.md"))
                self.assertEqual((args.titulo, args.materia, args.docente), ("T", "M", "D"))

    def test_copia_can_be_repeated(self):
        from investigacion.cli import build_parser

        args = build_parser().parse_args(
            ["t.md", "--titulo", "T", "--materia", "M", "--docente", "D", "--copia", "a", "--copia", "b"]
        )
        self.assertEqual(args.copia, [Path("a"), Path("b")])

    def test_copia_defaults_to_none(self):
        from investigacion.cli import build_parser

        args = build_parser().parse_args(["t.md", "--titulo", "T", "--materia", "M", "--docente", "D"])
        self.assertIsNone(args.copia)

    def test_integrantes_is_optional_and_takes_one_argument(self):
        from investigacion.cli import build_parser

        parser = build_parser()
        base = ["t.md", "--titulo", "T", "--materia", "M", "--docente", "D"]
        self.assertIsNone(parser.parse_args(base).integrantes)
        args = parser.parse_args(base + ["--integrantes", "Ana Ruiz, Luis Paz"])
        self.assertEqual(args.integrantes, "Ana Ruiz, Luis Paz")

    def test_grupo_is_optional(self):
        from investigacion.cli import build_parser

        parser = build_parser()
        base = ["t.md", "--titulo", "T", "--materia", "M", "--docente", "D"]
        self.assertIsNone(parser.parse_args(base).grupo)
        self.assertEqual(parser.parse_args(base + ["--grupo", "7-A"]).grupo, "7-A")


class RequiredEnvTests(unittest.TestCase):
    """ALUMNO e INTEGRANTES son opcionales; el resto sigue siendo obligatorio."""

    def test_alumno_is_not_required(self):
        from investigacion.cli import REQUIRED_ENV

        self.assertNotIn("ALUMNO", REQUIRED_ENV)
        self.assertEqual(set(REQUIRED_ENV), {"UNIVERSIDAD", "FACULTAD", "SEMESTRE"})

    def test_cli_does_not_complain_about_a_missing_alumno(self):
        import io
        from contextlib import redirect_stderr
        from unittest.mock import patch

        from investigacion.cli import main

        environment = {"UNIVERSIDAD": "U", "FACULTAD": "F", "SEMESTRE": "7"}
        with TemporaryDirectory() as directory:
            argv = [
                str(Path(directory) / "no-existe.md"),
                "--titulo", "T", "--materia", "M", "--docente", "D",
                "--env-file", str(Path(directory) / "sin-env"),
            ]
            stderr = io.StringIO()
            with patch.dict("os.environ", environment, clear=True):
                with redirect_stderr(stderr):
                    self.assertEqual(main(argv), 1)
        # Falla por el Markdown inexistente, no por faltar ALUMNO.
        self.assertNotIn("Faltan variables", stderr.getvalue())
        self.assertIn("no-existe.md", stderr.getvalue())


class CopyTests(unittest.TestCase):
    def test_copies_the_pdf_to_each_directory(self):
        from investigacion.generator import copy_pdf_to

        with TemporaryDirectory() as directory:
            base = Path(directory)
            pdf = base / "salida" / "trabajo.pdf"
            pdf.parent.mkdir()
            pdf.write_bytes(b"%PDF-1.5 contenido")
            copies = copy_pdf_to(pdf, [base / "usb", base / "nube" / "materia"])
            self.assertEqual(len(copies), 2)
            for copy in copies:
                self.assertEqual(copy.name, "trabajo.pdf")
                self.assertEqual(copy.read_bytes(), b"%PDF-1.5 contenido")

    def test_skips_the_main_output_directory_and_duplicates(self):
        from investigacion.generator import copy_pdf_to

        with TemporaryDirectory() as directory:
            base = Path(directory)
            pdf = base / "trabajo.pdf"
            pdf.write_bytes(b"pdf")
            copies = copy_pdf_to(pdf, [base, base / "usb", base / "usb"])
            self.assertEqual(len(copies), 1)

    def test_rejects_a_destination_that_is_a_file(self):
        from investigacion.generator import copy_pdf_to

        with TemporaryDirectory() as directory:
            base = Path(directory)
            pdf = base / "trabajo.pdf"
            pdf.write_bytes(b"pdf")
            ocupado = base / "ocupado.txt"
            ocupado.write_text("x", encoding="utf-8")
            with self.assertRaises(GenerationError):
                copy_pdf_to(pdf, [ocupado])


HAS_TOOLCHAIN = bool(shutil.which("pandoc") and shutil.which("pdflatex"))

# Cada bloque cubre un elemento de Markdown que necesita macros propias en la
# plantilla. Si la plantilla pierde alguna, la compilacion falla aqui.
RICH_MARKDOWN = """# Introducción

Texto con `codigo`, ~~tachado~~ y una fórmula $E = mc^2$.

$$\\int_0^1 x^2\\,dx = \\frac{1}{3}$$

# Desarrollo

| Tipo | Total | Unitario |
|---|---|---|
| Fijo | Constante | Decrece |
| Variable | Proporcional | Constante |

```python
def f(x):
    return x
```

Una nota al pie[^1] y una cita.

[^1]: Contenido de la nota.

> Cita en bloque.

# Conclusión

Cierre con símbolos 100% & seguros.

# Referencias

Autor, A. (2026). *Título*. Editorial.
"""


# Cada elemento de aqui fallaba antes: el tachado dentro de una tabla dejaba a
# pdflatex dando vueltas para siempre, el diagrama de caja salia como [?][?][?]
# y el resaltado, el emoji y el <br> no llegaban al PDF.
EXTENDED_MARKDOWN = """# Introducción

Texto ==resaltado==, ~~tachado~~, H~2~O, X^2^, <mark>marcado</mark>,
<u>subrayado</u>, <kbd>Ctrl</kbd> y un salto<br>de linea.

Emoji pegado 🚀 y por código :joy:. URL suelta https://example.com

# Desarrollo

| Elemento | Estado |
|---|---|
| ~~tachado~~ | ==resaltado== |

```
  Inicio
      └──► Paso
              └──► Fin
        ┌─────┬─────┐
        │     │     │
        └─────┴─────┘
        ╭─────╮
        ╰─────╯
```

- [x] Tarea hecha
- [ ] Tarea pendiente

Término
: Definición del término.

# Conclusión

Cierre.

# Referencias

Autor, A. (2026). *Título*. Editorial.
"""


@unittest.skipUnless(bool(shutil.which("pandoc")), "Requiere pandoc instalado")
class MarkdownSyntaxTests(unittest.TestCase):
    """La sintaxis extendida de Markdown tiene que llegar a LaTeX, no perderse."""

    def convert(self, markdown: str, warnings: list[str] | None = None) -> str:
        from investigacion.generator import pandoc_to_latex

        with TemporaryDirectory() as directory:
            source = Path(directory) / "trabajo.md"
            source.write_text(markdown, encoding="utf-8")
            return pandoc_to_latex(
                source, markdown, on_warning=warnings.append if warnings is not None else None
            )

    def test_highlight_strikethrough_and_scripts(self):
        latex = self.convert("==resaltado== ~~tachado~~ H~2~O X^2^\n")
        self.assertIn(r"\hl{resaltado}", latex)
        self.assertIn(r"\st{tachado}", latex)
        self.assertIn(r"\textsubscript{2}", latex)
        self.assertIn(r"\textsuperscript{2}", latex)

    def test_emoji_by_shortcode_becomes_the_character(self):
        self.assertIn("🚀", self.convert("Despegue :rocket:\n"))

    def test_a_bare_url_becomes_a_link(self):
        self.assertIn(r"\url{https://example.com}", self.convert("Ver https://example.com\n"))

    def test_inline_html_is_translated_instead_of_discarded(self):
        latex = self.convert(
            "Uno<br>dos <mark>marca</mark> <u>subrayado</u> <sub>b</sub> <del>viejo</del>\n"
        )
        self.assertIn("\\\\", latex)              # <br> es un salto de linea de LaTeX
        self.assertIn(r"\hl{marca}", latex)
        self.assertIn(r"\ul{subrayado}", latex)
        self.assertIn(r"\textsubscript{b}", latex)
        self.assertIn(r"\st{viejo}", latex)

    def test_html_inside_a_code_block_is_left_alone(self):
        latex = self.convert("```html\n<mark>literal</mark>\n```\n")
        self.assertNotIn(r"\hl{", latex)

    def test_a_fenced_div_becomes_a_marked_box(self):
        latex = self.convert("::: nota\nOjo con esto.\n:::\n")
        self.assertIn(r"\begin{CajaMarcada}{Nota}", latex)
        self.assertIn(r"\end{CajaMarcada}", latex)

    def test_a_box_accepts_its_own_title(self):
        latex = self.convert('::: {.aviso title="Antes de entregar"}\nRevisa.\n:::\n')
        self.assertIn(r"\begin{CajaMarcada}{Antes de entregar}", latex)

    def test_the_references_get_a_hanging_indent(self):
        latex = self.convert(
            "# Referencias\n\n- Autor, A. (2020). *Titulo*. Editorial.\n"
        )
        self.assertIn(r"\begin{ReferenciasAPA}", latex)
        self.assertIn(r"\end{ReferenciasAPA}", latex)
        # La lista se deshace: una referencia por parrafo, no una vineta.
        self.assertNotIn(r"\begin{itemize}", latex)

    def test_an_image_that_cannot_be_downloaded_does_not_break_the_document(self):
        """El PDF se genera igual: queda el texto alternativo y un aviso."""

        avisos: list[str] = []
        latex = self.convert("![Un logo](https://localhost:1/no-existe.png)\n", avisos)
        self.assertNotIn("localhost:1", latex)
        self.assertIn(r"\emph{Un logo}", latex)
        self.assertTrue(any("no-existe.png" in aviso for aviso in avisos))
        # Sin imagen no queda una figura con su pie huerfano.
        self.assertNotIn(r"\caption", latex)

    def test_a_missing_local_image_is_reported_instead_of_failing(self):
        avisos: list[str] = []
        latex = self.convert("![Un diagrama](no-esta.png)\n", avisos)
        self.assertIn(r"\emph{Un diagrama}", latex)
        self.assertTrue(any("no-esta.png" in aviso for aviso in avisos))

    @unittest.skipUnless(bool(shutil.which("dot")), "Requiere Graphviz instalado")
    def test_a_dot_block_becomes_a_drawn_diagram(self):
        latex = self.convert(
            '```{.dot caption="Un arbol"}\ndigraph { a -> b; }\n```\n'
        )
        self.assertIn(r"\includegraphics", latex)
        self.assertIn("diagrams/", latex)
        self.assertNotIn("digraph", latex)

    @unittest.skipIf(bool(shutil.which("dot")), "Comprueba el caso sin Graphviz")
    def test_without_graphviz_the_diagram_stays_as_code(self):
        avisos: list[str] = []
        latex = self.convert("```dot\ndigraph { a -> b; }\n```\n", avisos)
        self.assertIn("digraph", latex)
        self.assertTrue(any("Graphviz" in aviso for aviso in avisos))

    def test_a_pgfplot_block_becomes_a_figure(self):
        latex = self.convert(
            '```{.pgfplot caption="Horas"}\n'
            "\\begin{axis}[ybar]\n  \\addplot coordinates {(A,1)};\n\\end{axis}\n```\n"
        )
        self.assertIn(r"\begin{tikzpicture}", latex)
        self.assertIn(r"\begin{axis}[ybar]", latex)
        self.assertIn(r"\caption{Horas}", latex)
        # No debe salir como codigo: es una figura.
        self.assertNotIn(r"\begin{Shaded}", latex)

    def test_a_tikz_block_is_not_wrapped_twice(self):
        latex = self.convert(
            "```tikz\n\\begin{tikzpicture}\n\\draw (0,0) -- (1,1);\n"
            "\\end{tikzpicture}\n```\n"
        )
        self.assertEqual(latex.count(r"\begin{tikzpicture}"), 1)

    def test_task_lists_and_definition_lists(self):
        latex = self.convert("- [x] hecha\n- [ ] pendiente\n\nTermino\n: Definicion\n")
        self.assertIn(r"\boxtimes", latex)
        self.assertIn(r"\square", latex)
        self.assertIn(r"\begin{description}", latex)


@unittest.skipUnless(HAS_TOOLCHAIN, "Requiere pandoc y pdflatex instalados")
class PdfGenerationTests(unittest.TestCase):
    def test_document_with_tables_and_code_compiles(self):
        from investigacion.generator import generate_pdf

        data = DocumentData("U", "F", "A", "2026-2", "Prueba", "M", "D", "Agosto 23, 2026")
        with TemporaryDirectory() as directory:
            source = Path(directory) / "trabajo.md"
            source.write_text(RICH_MARKDOWN, encoding="utf-8")
            output_pdf = generate_pdf(source, Path(directory) / "salida", data)
            self.assertTrue(output_pdf.is_file())
            self.assertGreater(output_pdf.stat().st_size, 0)

    def test_extended_syntax_compiles(self):
        """Regresion: el tachado en una tabla colgaba pdflatex sin llegar a fallar."""

        from investigacion.generator import generate_pdf

        data = DocumentData("U", "F", "A", "2026-2", "Extendida", "M", "D", "Agosto 23, 2026")
        with TemporaryDirectory() as directory:
            source = Path(directory) / "trabajo.md"
            source.write_text(EXTENDED_MARKDOWN, encoding="utf-8")
            avisos: list[str] = []
            output_pdf = generate_pdf(
                source, Path(directory) / "salida", data, on_warning=avisos.append
            )
            self.assertTrue(output_pdf.is_file())
            # Ningun simbolo del documento debe acabar como [?] en el PDF.
            self.assertEqual(avisos, [])

    def test_backslashes_in_the_text_do_not_break_the_build(self):
        from investigacion.generator import generate_pdf

        data = DocumentData("U", "F", "A", "2026-2", "Rutas", "M", "D", "Agosto 23, 2026")
        with TemporaryDirectory() as directory:
            source = Path(directory) / "trabajo.md"
            source.write_text("# Introducción\n\nRuta C:\\Users\\alumno\n", encoding="utf-8")
            self.assertTrue(generate_pdf(source, Path(directory) / "salida", data).is_file())


class TuiTests(unittest.TestCase):
    def test_build_argv_omits_empty_and_repeats_copies(self):
        argv = build_argv(
            {"markdown": "a.md", "titulo": "T", "materia": "M", "docente": "",
             "copia": "x; y", "permitir_latex": "si"}
        )
        self.assertEqual(
            argv,
            ["a.md", "--titulo", "T", "--materia", "M", "--copia", "x", "--copia", "y", "--permitir-latex"],
        )

    def test_generate_requires_mandatory_fields(self):
        answers = iter(["g", "s"])
        with patch("investigacion.tui.cli_main") as fake:
            tui_run(lambda _p: next(answers), lambda *a: None)
        fake.assert_not_called()

    def test_full_flow_calls_cli(self):
        answers = iter(["1", "a.md", "2", "T", "3", "M", "g", "", "s"])
        with patch("investigacion.tui.cli_main", return_value=0) as fake:
            tui_run(lambda _p: next(answers), lambda *a: None)
        fake.assert_called_once_with(["a.md", "--titulo", "T", "--materia", "M"])

    def test_menu_marks_missing_fields_and_shows_help(self):
        output = []
        answers = iter(["2", "T", "s"])
        tui_run(lambda _p: next(answers), lambda *a: output.append(" ".join(map(str, a))))
        text = "\n".join(output)
        self.assertIn("[FALTA]", text)
        self.assertIn("Ejemplo:", text)
        self.assertIn("Falta: Archivo Markdown, Materia", text)

    def test_menu_says_ready_when_required_fields_are_filled(self):
        output = []
        answers = iter(["1", "a.md", "2", "T", "3", "M", "s"])
        tui_run(lambda _p: next(answers), lambda *a: output.append(" ".join(map(str, a))))
        self.assertIn("Listo. Escribe g", output[-1])


if __name__ == "__main__":
    unittest.main()
