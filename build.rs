
#[macro_use]
extern crate clap;
extern crate regex;

use clap::Shell;

include!("src/cli.rs");

fn main() {
    let mut app = build_cli();
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    app.gen_completions("multitest", Shell::Bash, &out_dir);
    app.gen_completions("multitest", Shell::Fish, &out_dir);
    app.gen_completions("multitest", Shell::Zsh, &out_dir);
}
