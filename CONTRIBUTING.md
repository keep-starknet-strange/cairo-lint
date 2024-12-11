<h1 style="border-bottom: 0;">Contribution Guideline</h1>


## Environment setup

- Install the latest Rust via [rustup](https://doc.rust-lang.org/cargo/getting-started/installation.html).
- Install the latest [Scarb via ASDF](https://docs.swmansion.com/scarb/download.html#install-via-asdf).

## Contributing

- Before you open a pull request, it is always a good idea to search the issues and verify if the feature you would like
to add hasn't been already discussed.
- We also appreciate creating a feature request before making a contribution, so it can be discussed before you get to
work.
- If the change you are introducing is changing or breaking the behavior of any already existing features, make sure to
include that information in the pull request description.

## Testing

### Running tests

To run the tests you'll need to provide the path to the cairo corelib (at some point this should be automated but we're
not there yet).

```sh
CORELIB_PATH="/path/to/corelib/src" cargo test
```

### CLI instructions

To add a new test you can use the dev cli with:

```
cargo run --bin create_test <lint_name>
```

### Manual instructions

Each lint should have its own tests and should be extensive. To create a new test for a lint you need to create a file
in the [test_files folder](./crates/cairo-lint-core/tests/test_files/) and should be named as your lint. The file should
have this format:

```rust
//! > Test name

//! > cairo_code
fn main() {
    let a: Option<felt252> = Option::Some(1);
}
```

Then in the [test file](crates/cairo-lint-core/tests/tests.rs) declare your lint like so:

```rust
test_file!(if_let, "Test name");
```

The first argument is the lint name (also the file name) and the other ones are the test names. After that you can run

```
FIX_TESTS=1 cargo test -p cairo-lint-core <name_of_lint>
```

This will generate the expected values in your test file. Make sure it is correct.
