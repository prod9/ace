import { readdir, mkdir, cp } from "node:fs/promises";
import { join, basename, extname } from "node:path";

const ROOT = import.meta.dir;
const PAGES_DIR = join(ROOT, "pages");
const PARTIALS_DIR = join(ROOT, "partials");
const STATIC_DIR = join(ROOT, "static");
const DIST_DIR = join(ROOT, "dist");
const LAYOUT_PATH = join(ROOT, "layout.html");

// -- title derivation --

function titleFromFilename(filename: string): string {
  const name = basename(filename, extname(filename));
  if (name === "index") return "Accelerated Coding Environment";
  return name.charAt(0).toUpperCase() + name.slice(1);
}

// -- directive expansion --
// All directives use <!-- @name [arg] --> syntax.

async function expand(html: string, vars: Record<string, string>): Promise<string> {
  let result = "";
  let pos = 0;

  while (pos < html.length) {
    const start = html.indexOf("<!-- @", pos);
    if (start === -1) {
      result += html.slice(pos);
      break;
    }

    result += html.slice(pos, start);

    const end = html.indexOf("-->", start);
    if (end === -1) {
      result += html.slice(pos);
      break;
    }

    const directive = html.slice(start + 6, end).trim();
    pos = end + 3;

    if (directive.startsWith("include ")) {
      const file = directive.slice(8).trim();
      result += await Bun.file(join(PARTIALS_DIR, file)).text();
    } else if (vars[directive] !== undefined) {
      result += vars[directive];
    } else {
      result += html.slice(start, pos);
    }
  }

  return result;
}

// -- build --

async function build() {
  const layout = await Bun.file(LAYOUT_PATH).text();

  await mkdir(DIST_DIR, { recursive: true });

  const files = await readdir(PAGES_DIR);
  const pages = files.filter((f) => f.endsWith(".html"));

  for (const page of pages) {
    const content = await Bun.file(join(PAGES_DIR, page)).text();
    const title = titleFromFilename(page);

    const output = await expand(layout, { title, content });

    await Bun.write(join(DIST_DIR, page), output);
    console.log(`  ${page}`);
  }

  await cp(STATIC_DIR, DIST_DIR, { recursive: true });
  console.log("  static/");

  console.log(`\n  ${pages.length} pages -> dist/`);
}

build();
