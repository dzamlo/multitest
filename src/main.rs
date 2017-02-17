extern crate termion;

#[macro_use]
mod eprint;
mod test;

use std::ffi::OsStr;
use std::process::exit;
use test::Test;

fn run_tests<T1: AsRef<OsStr>,
             T2: AsRef<OsStr>,
             T3: AsRef<OsStr>,
             T: IntoIterator<Item = Test<T1, T2, T3>>>
    (tests: T)
     -> bool {

    let mut successes = vec![];
    let mut failures = vec![];

    for test in tests {
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

    if total == 0 {
        eprintln_red!("No tests found")
    }

    failures.is_empty() && total > 0
}

fn main() {
    let test1 = Test::new("test1",
                          vec!["env".to_string()],
                          vec![("TARGET".to_string(), "TEST".to_string())]);
    let test2 = Test::new("test2",
                          vec!["true".to_string()],
                          vec![("TARGET".to_string(), "TEST".to_string())]);
    let test3 = Test::new("test3",
                          vec!["false".to_string()],
                          vec![("TARGET".to_string(), "TEST".to_string())]);
    let test4 = Test::new("test4",
                          vec!["command_that_dont_exist".to_string()],
                          vec![("TARGET".to_string(), "TEST".to_string())]);

    let tests = vec![test1, test2, test3, test4];
    let success = run_tests(tests);
    exit(if success { 0 } else { 1 });
}
