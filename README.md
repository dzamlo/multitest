# `multitest`

`multitest` is a tool to run multiple tests with a nice output.


## Installation


You can install the latest version from the git repository:

```bash
cargo install --git https://github.com/dzamlo/multitest.git
```

If you use Arch Linux, a `PKGBUILD` file is present in the `aur` directory.

bash, zsh and fish completion files are also generated but not installed with `cargo install`. To use them you need to either use the `PKGBUILD` file, or manually build the application yourself and copy the files.

## Usage

You must first describe your tests in a `multitest.toml` file at the root of your project.

By default, `multitest` will try to find a `multitest.toml` file in the current directory and its parent:
```bash
multitest
```

You can also specify a configuration file on the command line:
```bash
multitest --config multitest-demo.toml
```


The configuration file contains a description of the tests to run.

A test is generated for each element of the Cartesian product of its variables.

The test name, args, and environment variables names and values use [liquid].

[liquid]: http://liquidmarkup.org/

You can look at the provided `multitest.toml` and `multitest-demo.toml` for some examples.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
