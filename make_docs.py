import html
from pathlib import Path

DOCS_DIR = Path(__file__).parent / "docs"
DOCS_DIR.mkdir(parents=True, exist_ok=True)

index_content = """\
<h1>Binary builds</h1>
<ul>
"""

for file in (Path(__file__).parent / "target/release").iterdir():
    file.rename(DOCS_DIR / file.name)
    link = html.escape(file.name)
    index_content += f'<li><a href="{link}">{link}</a></li>\n'

index_content += "</ul>\n"

(DOCS_DIR / "index.html").write_text(index_content, "utf-8")
