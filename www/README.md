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

Build first:

```sh
cd www
bun run build
```

Publish separately:

```sh
cd www
./publish.sh
```

`publish.sh` does not run the build for you. It expects `dist/` to already contain the site you
want to publish.

Internally it:

1. stages `www/dist`
2. creates a temporary commit object from the staged site output
3. runs `git subtree split --prefix=www/dist`
4. updates the local `gh-pages` branch to that split commit
5. force-pushes that commit to `origin/gh-pages`

This keeps `gh-pages` as generated output only, with no manual file copying between branches.

## Next Step

Before first public publish, update the page content so it matches the current CLI, backend set,
and install story.
