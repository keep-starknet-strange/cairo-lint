<div align="center">
<h1>
    cairo-lint
    <br>

[![Check Workflow Status](https://github.com/keep-starknet-strange/snos/actions/workflows/check.yml/badge.svg)](https://github.com/keep-starknet-strange/snos/actions/workflows/check.yml)
[![Telegram](https://img.shields.io/badge/telegram-cairolint-blue.svg?logo=telegram)](https://t.me/cairolint)

[![Exploration_Team](https://img.shields.io/badge/Exploration_Team-29296E.svg?&style=for-the-badge&logo=data:image/svg%2bxml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiPz48c3ZnIGlkPSJhIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxODEgMTgxIj48ZGVmcz48c3R5bGU+LmJ7ZmlsbDojZmZmO308L3N0eWxlPjwvZGVmcz48cGF0aCBjbGFzcz0iYiIgZD0iTTE3Ni43Niw4OC4xOGwtMzYtMzcuNDNjLTEuMzMtMS40OC0zLjQxLTIuMDQtNS4zMS0xLjQybC0xMC42MiwyLjk4LTEyLjk1LDMuNjNoLjc4YzUuMTQtNC41Nyw5LjktOS41NSwxNC4yNS0xNC44OSwxLjY4LTEuNjgsMS44MS0yLjcyLDAtNC4yN0w5Mi40NSwuNzZxLTEuOTQtMS4wNC00LjAxLC4xM2MtMTIuMDQsMTIuNDMtMjMuODMsMjQuNzQtMzYsMzcuNjktMS4yLDEuNDUtMS41LDMuNDQtLjc4LDUuMThsNC4yNywxNi41OGMwLDIuNzIsMS40Miw1LjU3LDIuMDcsOC4yOS00LjczLTUuNjEtOS43NC0xMC45Ny0xNS4wMi0xNi4wNi0xLjY4LTEuODEtMi41OS0xLjgxLTQuNCwwTDQuMzksODguMDVjLTEuNjgsMi4zMy0xLjgxLDIuMzMsMCw0LjUzbDM1Ljg3LDM3LjNjMS4zNiwxLjUzLDMuNSwyLjEsNS40NCwxLjQybDExLjQtMy4xMSwxMi45NS0zLjYzdi45MWMtNS4yOSw0LjE3LTEwLjIyLDguNzYtMTQuNzYsMTMuNzNxLTMuNjMsMi45OC0uNzgsNS4zMWwzMy40MSwzNC44NGMyLjIsMi4yLDIuOTgsMi4yLDUuMTgsMGwzNS40OC0zNy4xN2MxLjU5LTEuMzgsMi4xNi0zLjYsMS40Mi01LjU3LTEuNjgtNi4wOS0zLjI0LTEyLjMtNC43OS0xOC4zOS0uNzQtMi4yNy0xLjIyLTQuNjItMS40Mi02Ljk5LDQuMyw1LjkzLDkuMDcsMTEuNTIsMTQuMjUsMTYuNzEsMS42OCwxLjY4LDIuNzIsMS42OCw0LjQsMGwzNC4zMi0zNS43NHExLjU1LTEuODEsMC00LjAxWm0tNzIuMjYsMTUuMTVjLTMuMTEtLjc4LTYuMDktMS41NS05LjE5LTIuNTktMS43OC0uMzQtMy42MSwuMy00Ljc5LDEuNjhsLTEyLjk1LDEzLjg2Yy0uNzYsLjg1LTEuNDUsMS43Ni0yLjA3LDIuNzJoLS42NWMxLjMtNS4zMSwyLjcyLTEwLjYyLDQuMDEtMTUuOGwxLjY4LTYuNzNjLjg0LTIuMTgsLjE1LTQuNjUtMS42OC02LjA5bC0xMi45NS0xNC4xMmMtLjY0LS40NS0xLjE0LTEuMDgtMS40Mi0xLjgxbDE5LjA0LDUuMTgsMi41OSwuNzhjMi4wNCwuNzYsNC4zMywuMTQsNS43LTEuNTVsMTIuOTUtMTQuMzhzLjc4LTEuMDQsMS42OC0xLjE3Yy0xLjgxLDYuNi0yLjk4LDE0LjEyLTUuNDQsMjAuNDYtMS4wOCwyLjk2LS4wOCw2LjI4LDIuNDYsOC4xNiw0LjI3LDQuMTQsOC4yOSw4LjU1LDEyLjk1LDEyLjk1LDAsMCwxLjMsLjkxLDEuNDIsMi4wN2wtMTMuMzQtMy42M1oiLz48L3N2Zz4=)](https://github.com/keep-starknet-strange)

</h1>
</div>

A collection of lints to catch common mistakes and improve your [Cairo](https://github.com/starkware-libs/cairo) code.

## Usage

cairo-lint can either be used as a library or as a standalone binary. It can either just detect or fix the detected
problems.

To use it with scarb simply install it like so:

```sh
cargo install scarb-cairo-lint --git https://github.com/keep-starknet-strange/cairo-lint
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

Note: You can also include test files with the `--test` flag

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/0xLucqs"><img src="https://avatars.githubusercontent.com/u/70894690?v=4?s=100" width="100px;" alt="0xLucqs"/><br /><sub><b>0xLucqs</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=0xLucqs" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/coxmars"><img src="https://avatars.githubusercontent.com/u/75222804?v=4?s=100" width="100px;" alt="Marco Araya Jiménez"/><br /><sub><b>Marco Araya Jiménez</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=coxmars" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=coxmars" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/enitrat"><img src="https://avatars.githubusercontent.com/u/60658558?v=4?s=100" width="100px;" alt="Mathieu"/><br /><sub><b>Mathieu</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=enitrat" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=enitrat" title="Tests">⚠️</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=enitrat" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/lauchaves"><img src="https://avatars.githubusercontent.com/u/5482929?v=4?s=100" width="100px;" alt="Lau Chaves"/><br /><sub><b>Lau Chaves</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=lauchaves" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=lauchaves" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/renzobanegass"><img src="https://avatars.githubusercontent.com/u/55169794?v=4?s=100" width="100px;" alt="Renzo Banegas"/><br /><sub><b>Renzo Banegas</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=renzobanegass" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=renzobanegass" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/stevencartavia"><img src="https://avatars.githubusercontent.com/u/112043913?v=4?s=100" width="100px;" alt="Steven"/><br /><sub><b>Steven</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=stevencartavia" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=stevencartavia" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/lindsaymoralesb"><img src="https://avatars.githubusercontent.com/u/87027508?v=4?s=100" width="100px;" alt="Lindsay Morales"/><br /><sub><b>Lindsay Morales</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=lindsaymoralesb" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=lindsaymoralesb" title="Tests">⚠️</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/BernalHQ"><img src="https://avatars.githubusercontent.com/u/17929742?v=4?s=100" width="100px;" alt="Bernal Hidalgo"/><br /><sub><b>Bernal Hidalgo</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=BernalHQ" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=BernalHQ" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/chachaleo"><img src="https://avatars.githubusercontent.com/u/49371958?v=4?s=100" width="100px;" alt="Charlotte"/><br /><sub><b>Charlotte</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=chachaleo" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=chachaleo" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/BrunoAmbricca"><img src="https://avatars.githubusercontent.com/u/64877723?v=4?s=100" width="100px;" alt="Bruno Ambricca"/><br /><sub><b>Bruno Ambricca</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=BrunoAmbricca" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=BrunoAmbricca" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/guha-rahul"><img src="https://avatars.githubusercontent.com/u/52607971?v=4?s=100" width="100px;" alt="guha-rahul"/><br /><sub><b>guha-rahul</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=guha-rahul" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=guha-rahul" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/manlikeHB"><img src="https://avatars.githubusercontent.com/u/109147010?v=4?s=100" width="100px;" alt="Yusuf Habib"/><br /><sub><b>Yusuf Habib</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=manlikeHB" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=manlikeHB" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/TropicalDog17"><img src="https://avatars.githubusercontent.com/u/79791913?v=4?s=100" width="100px;" alt="Tuan Tran"/><br /><sub><b>Tuan Tran</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=TropicalDog17" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=TropicalDog17" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/JoE11-y"><img src="https://avatars.githubusercontent.com/u/55321462?v=4?s=100" width="100px;" alt="BlockyJ"/><br /><sub><b>BlockyJ</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=JoE11-y" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=JoE11-y" title="Tests">⚠️</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="http://linkedin.com/in/luis-jimenez22"><img src="https://avatars.githubusercontent.com/u/87153882?v=4?s=100" width="100px;" alt="Luis Jiménez"/><br /><sub><b>Luis Jiménez</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=jimenezz22" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=jimenezz22" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/MariangelaNM"><img src="https://avatars.githubusercontent.com/u/91926755?v=4?s=100" width="100px;" alt="Mariángela N."/><br /><sub><b>Mariángela N.</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=MariangelaNM" title="Code">💻</a> <a href="https://github.com/keep-starknet-strange/cairo-lint/commits?author=MariangelaNM" title="Tests">⚠️</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## Contributing

**[Join the telegram group](t.me/cairo-lint)**

To run the tests you'll need to provide the path to the cairo corelib (at some point this should be automated but we're
not there yet).

```sh
CORELIB_PATH="/path/to/corelib/src" cargo test
```

### Cli instructions

To add a new test you can use the dev cli with:

```bash
cargo run --bin create_test <lint_name>
```

### Manual instructions

Each lint should have its own tests and should be extensive. To create a new test for a lint you need to create a file
in the [test_files folder](./crates/cairo-lint-core/tests/test_files/) and should be named as your lint. The file should
have this format:

```txt
//! > Test name

//! > cairo_code
fn main() {
    let a: Option<felt252> = Option::Some(1);
}
```

Then in the [test file](crates/cairo-lint-core/tests/tests.rs) declare your lint like so:

```rs
test_file!(if_let, "Test name");
```

The first argument is the lint name (also the file name) and the other ones are the test names. After that you can run

```
FIX_TESTS=1 cargo test -p cairo-lint-core <name_of_lint>
```

This will generate the expected values in your test file. Make sure it is correct.
