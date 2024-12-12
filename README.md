# Cairo lint

A collection of lints to catch common mistakes and improve your [Cairo](https://github.com/starkware-libs/cairo) code.

## Quick start

cairo-lint can either be used as a library or as a standalone binary. It can either just detect or fix the detected
problems.

To use it with [scarb](https://github.com/software-mansion/scarb) simply install it like so:

```sh
cargo install scarb-cairo-lint --git https://github.com/software-mansion/cairo-lint
```

and then either run:

```sh
# Checks for bad patterns
scarb cairo-lint
```

```sh
# Checks and fixes what it can
scarb cairo-lint --fix
```

## Features

- The `--test` flag to include test files.

## Community

As for now there is only a [telegram channel](https://t.me/cairolint) dedicated to cairo-lint.
