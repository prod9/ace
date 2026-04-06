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
want to publish, and it assumes those `www/dist` changes are already committed on `main`.

Internally it:

1. verifies you are publishing from `main`
2. verifies `www/dist` has no uncommitted changes
3. runs `git subtree push --prefix=www/dist --rejoin gh gh-pages`

This keeps `publish.sh` focused on publishing only. Build and commit the generated site output
before running it. `--rejoin` may add the subtree join commit on `main` so later publishes do
not need to recompute the split from scratch.

## Next Step

Before first public publish, update the page content so it matches the current CLI, backend set,
and install story.
