extern crate atty;
#[macro_use]
extern crate clap;
extern crate glob;
extern crate liquid;
extern crate regex;
extern crate shell_escape;
extern crate termcolor;
extern crate toml;

#[macro_use]
mod eprint;
mod cli;
mod config;
mod test;

use regex::Regex;
use std::process::exit;
use termcolor::ColorChoice;

fn main() {
    let matches = cli::build_cli().get_matches();

    let config_file = matches.value_of_os("config_file");
    let filter = matches
        .value_of("filter")
        .map(|filter| Regex::new(filter).unwrap());
    let color_choice = match matches.value_of("color").unwrap() {
        "always" => ColorChoice::Always,
        "auto" => if atty::is(atty::Stream::Stderr) {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        },
        "never" => ColorChoice::Never,
        _ => unreachable!(),
    };

    unsafe {
        eprint::set_color_choice(color_choice);
    }

    let success = match config::run_config_root(config_file, &filter) {
        Ok(result) => {
            result.summary();
            result.is_success()
        }
        Err(()) => false,
    };

    exit(if success { 0 } else { 1 });
}
