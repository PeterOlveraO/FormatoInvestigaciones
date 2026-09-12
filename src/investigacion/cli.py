"""Interfaz de linea de comandos para generar investigaciones."""

from __future__ import annotations

import argparse
import os
import sys
from datetime import date
from pathlib import Path

from .generator import (
    DocumentData,
    GenerationError,
    copy_pdf_to,
    format_delivery_date,
    generate_pdf,
    load_dotenv,
    parse_integrantes,
    read_markdown,
    resolve_markdown_path,
    validate_markdown,
)


# ALUMNO e INTEGRANTES son opcionales y pueden faltar los dos: en ese caso la
# portada omite esas lineas.
REQUIRED_ENV = {
    "UNIVERSIDAD": "universidad",
    "FACULTAD": "facultad",
    "SEMESTRE": "semestre",
}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="investigacion",
        description="Genera un trabajo academico APA en PDF desde un archivo Markdown.",
    )
    parser.add_argument("markdown", type=Path, help="Archivo .md con el contenido completo del trabajo")
    parser.add_argument("--titulo", required=True, help="Titulo del trabajo")
    parser.add_argument("--materia", required=True, help="Nombre de la materia")
    parser.add_argument(
        "--docente",
        default=None,
        help=(
            "Nombre del docente. Si se omite se usa la variable DOCENTE del .env; "
            "si tampoco esta, la linea no aparece en la portada."
        ),
    )
    parser.add_argument(
        "--integrantes",
        default=None,
        help=(
            "Nombres del equipo en un solo argumento, separados por comas: "
            '--integrantes "Ana Ruiz, Luis Paz". Aparecen uno por renglon en la '
            "portada. Si se omite, se usa la variable INTEGRANTES del .env."
        ),
    )
    parser.add_argument(
        "--grupo",
        default=None,
        help=(
            "Grupo de la materia, por ejemplo: --grupo \"7-A\". Aparece en la "
            "portada debajo de la materia. Si se omite, se usa la variable "
            "GRUPO del .env; si tampoco esta, la linea no aparece."
        ),
    )
    parser.add_argument(
        "--salida",
        type=Path,
        default=Path("output"),
        help="Directorio donde se guardara el PDF (por defecto: output)",
    )
    parser.add_argument(
        "--copia",
        type=Path,
        action="append",
        metavar="DIRECTORIO",
        help=(
            "Directorio adicional donde se guardara una copia del PDF. "
            "Se crea si no existe y puede repetirse para dejar varias copias."
        ),
    )
    parser.add_argument(
        "--env-file",
        type=Path,
        default=Path(".env"),
        help="Archivo dotenv con los datos permanentes (por defecto: .env)",
    )
    parser.add_argument(
        "--permitir-latex",
        action="store_true",
        help=(
            "Interpreta los comandos LaTeX escritos en el Markdown (\\newpage, etc.). "
            "Sin esta opcion las contrabarras se imprimen como texto y no rompen la compilacion."
        ),
    )
    parser.add_argument(
        "--plantilla",
        type=Path,
        default=None,
        help="Ruta alternativa a la plantilla LaTeX",
    )
    parser.add_argument(
        "--logos",
        type=Path,
        default=None,
        help=(
            "Directorio con los logos de la portada (logo-universidad.png y "
            "logo-facultad.png). Si se omite se usa la variable LOGOS del .env "
            "y, si tampoco esta, la carpeta logos/ junto a la plantilla."
        ),
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args, extras = parser.parse_known_args(argv)
    if extras:
        if any(not extra.startswith("-") for extra in extras):
            parser.error(
                "Sobran argumentos: "
                + " ".join(extras)
                + ". Si el titulo, la materia o el docente llevan espacios, escribelos entre comillas."
            )
        parser.error("Opciones no reconocidas: " + " ".join(extras))
    try:
        load_dotenv(args.env_file.expanduser().resolve())
        missing = [key for key in REQUIRED_ENV if not os.environ.get(key, "").strip()]
        if missing:
            names = ", ".join(missing)
            raise GenerationError(f"Faltan variables de entorno: {names}. Configura tu archivo .env.")

        markdown_path = resolve_markdown_path(args.markdown)
        markdown = read_markdown(markdown_path)
        for warning in validate_markdown(markdown):
            print(f"Advertencia: {warning}", file=sys.stderr)

        # --integrantes gana sobre la variable INTEGRANTES del .env, que existe
        # como respaldo para los equipos que siempre son los mismos.
        integrantes_raw = args.integrantes
        if integrantes_raw is None:
            integrantes_raw = os.environ.get("INTEGRANTES", "")

        # Mismo criterio para el grupo, que tambien suele repetirse todo el
        # semestre: la opcion gana sobre la variable del .env.
        grupo = args.grupo if args.grupo is not None else os.environ.get("GRUPO", "")

        # El docente cambia poco dentro de una materia, asi que tambien admite
        # respaldo en el .env; si no esta en ninguno, la portada omite la linea.
        docente = args.docente if args.docente is not None else os.environ.get("DOCENTE", "")

        # La carpeta de logos sigue el mismo patron: la opcion gana sobre la
        # variable del .env, que es donde conviene dejarla fija porque los logos
        # suelen vivir fuera del proyecto.
        logos = args.logos
        if logos is None and os.environ.get("LOGOS", "").strip():
            logos = Path(os.environ["LOGOS"].strip())

        data = DocumentData(
            universidad=os.environ["UNIVERSIDAD"].strip(),
            facultad=os.environ["FACULTAD"].strip(),
            alumno=os.environ.get("ALUMNO", "").strip(),
            semestre=os.environ["SEMESTRE"].strip(),
            titulo=args.titulo.strip(),
            materia=args.materia.strip(),
            docente=docente.strip(),
            fecha=format_delivery_date(date.today()),
            integrantes=parse_integrantes(integrantes_raw),
            grupo=grupo.strip(),
        )
        if not data.titulo or not data.materia:
            raise GenerationError("El titulo y la materia no pueden estar vacios.")
        output_pdf = generate_pdf(
            markdown_path,
            args.salida,
            data,
            args.plantilla,
            markdown=markdown,
            allow_raw_latex=args.permitir_latex,
            on_warning=lambda aviso: print(f"Advertencia: {aviso}", file=sys.stderr),
            logos_path=logos,
        )
        copies = copy_pdf_to(output_pdf, args.copia or [])
    except (GenerationError, OSError, UnicodeError) as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1

    print(f"PDF generado: {output_pdf}")
    for copy in copies:
        print(f"Copia guardada: {copy}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
