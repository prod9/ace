# ACE Presentation

Typst + Touying/Metropolis slides for ACE.

## Build

```bash
./build.sh              # compile to slides.pdf
./build.sh --watch      # recompile on change
```

Requires [Typst](https://typst.app/): `brew install typst`

## Structure

```
slides.typ          # entrypoint (includes sections)
theme.typ           # theme config + touying re-exports
sections/           # one file per section (a-f)
fonts/              # local font overrides
```
