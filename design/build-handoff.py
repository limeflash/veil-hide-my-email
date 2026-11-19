#!/usr/bin/env python3
"""Regenerate design/handoff/*.html from the design-canvas artboards.

The artboards in design/canvas/project/*.dc.html are the single source of the
markup. This strips the canvas wrapper and writes plain static HTML that uses
only tokens.css / components.css (and inpage.css), so the handoff always
matches the published design.

    python3 design/build-handoff.py
"""
import pathlib
import re

HERE = pathlib.Path(__file__).resolve().parent
SRC = HERE / "canvas" / "project"
OUT = HERE / "handoff"

PAGES = {
    # output file: (title, [artboards], [stylesheets])
    "popup.html": ("Veil — popup: all screens and states", ["Main", "PopupManage"], ["tokens.css", "components.css", "preview.css"]),
    "options.html": ("Veil — options page", ["Options"], ["tokens.css", "components.css", "preview.css"]),
    "welcome.html": ("Veil — welcome page", ["Welcome"], ["tokens.css", "components.css", "preview.css"]),
    "inpage.html": ("Veil — in-page autofill button", ["Inpage"], ["tokens.css", "components.css", "inpage.css", "preview.css"]),
    "components.html": ("Veil — brand, icon and component sampler", ["Brand"], ["tokens.css", "components.css", "preview.css"]),
}

NOTE = """<!--
  Static reference markup. Every state is a <section class="h-state" data-state="…">
  with an .h-state__label; port what is INSIDE the section (the .v-* / .veil-* tree).
  .h-* classes and preview.css are preview chrome only.
  Theme: follows the OS; force with <html data-theme="light|dark">.
-->"""


def body_of(name: str) -> str:
    text = (SRC / f"{name}.dc.html").read_text(encoding="utf-8")
    inner = text.split("</helmet>", 1)[1].split("</x-dc>", 1)[0].strip()
    inner = inner.replace(' data-theme="{{theme}}"', "")
    # drop the fixed artboard size from the board root
    inner = re.sub(r'(<div class="h-board[^"]*")\s+style="[^"]*"', r"\1", inner, count=1)
    return inner


for filename, (title, boards, sheets) in PAGES.items():
    links = "\n".join(f'<link rel="stylesheet" href="{s}">' for s in sheets)
    boards = [b for b in boards if (SRC / f"{b}.dc.html").exists()]
    if not boards:
        continue
    body = "\n\n".join(body_of(b) for b in boards)
    html = f"""<!doctype html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
{links}
</head>
<body class="v-root" style="margin:0">
{NOTE}
{body}
</body>
</html>
"""
    (OUT / filename).write_text(html, encoding="utf-8")
    print("wrote", filename, len(html))
