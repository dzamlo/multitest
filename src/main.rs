extern crate termion;

#[macro_use]
mod eprint;
mod test;
use test::Test;

fn main() {
    let test1 = Test::new("test1", vec!["env"], vec![("TARGET", "TEST")]);
    let test2 = Test::new("test2", vec!["true"], vec![("TARGET", "TEST")]);
    let test3 = Test::new("test3", vec!["false"], vec![("TARGET", "TEST")]);
    let test4 = Test::new("test4",
                          vec!["command_that_dont_exist"],
                          vec![("TARGET", "TEST")]);
    test1.run();
    test2.run();
    test3.run();
    test4.run();
}
