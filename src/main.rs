extern crate regex;
extern crate termion;
extern crate toml;

#[macro_use]
mod eprint;
mod config;
mod test;

use regex::Regex;
use std::env;
use std::ffi::OsStr;
use std::process::exit;
use test::Test;

fn run_tests<T1: AsRef<OsStr>,
             T2: AsRef<OsStr>,
             T3: AsRef<OsStr>,
             T: IntoIterator<Item = Test<T1, T2, T3>>>
    (tests: T,
     filter: Option<String>)
     -> bool {


    let filter = match filter {
        Some(filter) => {
            let regex = Regex::new(&*filter);
            match regex {
                Ok(regex) => Some(regex),
                Err(error) => {
                    eprintln_red!("Filter is not a valid regular expression: {}", error);
                    return false;
                }
            }
        }
        None => None,
    };

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


    let tests = config::load_config();

    let filter = env::args().skip(1).next();

    let success = match tests {
        Ok(tests) => run_tests(tests, filter),
        Err(()) => {
            false
        }
    };

    exit(if success { 0 } else { 1 });
}
