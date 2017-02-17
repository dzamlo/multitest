use std::ffi::OsStr;
use std::io;
use std::process::Command;
use std::process::ExitStatus;


pub struct Test<T1: AsRef<OsStr>, T2: AsRef<OsStr>, T3: AsRef<OsStr>> {
    pub name: String,
    pub args: Vec<T1>,
    pub env: Vec<(T2, T3)>,
}

impl<T1: AsRef<OsStr>, T2: AsRef<OsStr>, T3: AsRef<OsStr>> Test<T1, T2, T3> {
    pub fn new<S: Into<String>>(name: S, args: Vec<T1>, env: Vec<(T2, T3)>) -> Test<T1, T2, T3> {
        Test {
            name: name.into(),
            args: args,
            env: env,
        }
    }

    fn run_command(&self) -> io::Result<ExitStatus> {
        let mut command = Command::new(&self.args[0]);
        command.args(&self.args[1..]);
        for &(ref key, ref value) in &self.env {
            command.env(key, value);
        }

        command.status()


    }

    pub fn run(&self) -> bool {

        eprintln_bold!("Running test {}", self.name);

        let command_result = self.run_command();

        match command_result {
            Err(error) => {
                eprintln_red!("Test {} failed: {}", self.name, error);
                false

            }
            Ok(status) => {
                if status.success() {
                    eprintln_green!("Test {} was successful", self.name);
                    true
                } else {
                    match status.code() {
                        Some(code) => {
                            eprintln_red!("Test {} failed: exit code {}", self.name, code);
                        }
                        None => {
                            eprintln_red!("Test {} failed: no exit code", self.name);
                        }
                    }
                    false
                }
            }
        }
    }
}
