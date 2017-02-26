extern crate atty;
#[macro_use]
extern crate clap;
extern crate liquid;
extern crate regex;
extern crate termcolor;
extern crate toml;

#[macro_use]
mod eprint;
mod cli;
mod config;
mod test;

use regex::Regex;
use std::ffi::OsStr;
use std::process::exit;
use termcolor::ColorChoice;
use test::Test;

fn run_tests<T1: AsRef<OsStr>,
             T2: AsRef<OsStr>,
             T3: AsRef<OsStr>,
             T: IntoIterator<Item = Test<T1, T2, T3>>>
    (tests: T,
     filter: Option<Regex>)
     -> bool {
    let mut successes = vec![];
    let mut failures = vec![];
    let mut ignored = 0;

    for test in tests {

        if let Some(ref regex) = filter {
            if !regex.is_match(&*test.name) {
                ignored += 1;
                eprintln_bold!("Test {} ignored", test.name);
                continue;
            }
        }

        let test_success = test.run();
        if test_success {
            successes.push(test.name);
        } else {
            failures.push(test.name);
        }
    }

    eprintln_bold!("Summary");

    let total = successes.len() + failures.len();

    if !successes.is_empty() {
        eprintln_green!("Successes ({}/{}):", successes.len(), total);
        for success in &successes {
            eprintln_green!("  {}", success);
        }
    }


    if !failures.is_empty() {
        eprintln_red!("Failures ({}/{}):", successes.len(), total);
        for failure in &failures {
            eprintln_red!("  {}", failure);
        }
    }

    if ignored > 0 {
        eprintln_bold!("{} tests ignored", ignored);
    }

    if total == 0 {
        eprintln_red!("No tests executed")
    }

    failures.is_empty() && total > 0
}

fn main() {

    let matches = cli::build_cli().get_matches();

    let config_file = matches.value_of_os("config_file");
    let filter = matches.value_of("filter").map(|filter| Regex::new(filter).unwrap());
    let color_choice = match matches.value_of("color").unwrap() {
        "always" => ColorChoice::Always,
        "auto" => {
            if atty::is(atty::Stream::Stderr) {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            }
        }
        "never" => ColorChoice::Never,
        _ => unreachable!(),
    };

    unsafe {
        eprint::set_color_choice(color_choice);
    }

    let tests = config::load_config(config_file);


    let success = match tests {
        Ok(tests) => run_tests(tests, filter),
        Err(()) => {
            false
        }
    };

    exit(if success { 0 } else { 1 });
}
