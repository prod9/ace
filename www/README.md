# Website

This directory contains the static project site source and a very small custom build step.

## Layout

- `pages/` — page bodies. Each file becomes one output HTML file in `dist/`.
- `partials/` — shared HTML fragments included into pages, currently nav and footer.
- `static/` — static assets copied into `dist/` as-is.
- `layout.html` — outer HTML shell wrapped around every page.
- `build.ts` — Bun script that composes pages + layout + partials into `dist/`.
- `dist/` — generated site output. Safe to delete and rebuild.

## Build

Prerequisite: Bun installed.

```sh
cd www
bun run build
```

That runs `build.ts`, which:

1. Reads every `pages/*.html` file
2. Derives a page title from the filename
3. Expands simple directives in `layout.html`
4. Writes final HTML files to `dist/`
5. Copies `static/` into `dist/`

## Template Directives

The custom builder supports a tiny directive syntax:

- `<!-- @content -->` — inject current page body
- `<!-- @title -->` — inject derived page title
- `<!-- @include <file> -->` — include a file from `partials/`

Anything else is left unchanged.

## Output

The built site lives in `www/dist/` and is intended to be published directly as static files.

Current output files are flat:

- `dist/index.html`
- `dist/commands.html`
- `dist/configuration.html`
- `dist/backends.html`
- `dist/schools.html`
- `dist/site.css`

## Manual gh-pages Publish

Current plan is manual publish from the dev machine, not GitHub Actions.

Suggested flow:

```sh
cd www
bun run build

git switch gh-pages
rm -rf *
cp -R dist/* .
git add .
git commit -m "Publish site"
git push origin gh-pages
git switch main
```

Notes:

- Keep `gh-pages` as generated output only.
- Publish the contents of `dist/`, not the `dist/` directory itself.
- If this becomes repetitive, add a small helper script later rather than introducing CI yet.

## Next Step

Before first public publish, update the page content so it matches the current CLI, backend set,
and install story.
