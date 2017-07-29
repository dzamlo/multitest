use shell_escape::escape;
use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::process::Command;
use std::process::ExitStatus;

pub struct Test<T1, T2, T3> {
    pub name: String,
    pub command: Vec<T1>,
    pub env: Vec<(T2, T3)>,
    pub clear_env: bool,
}

impl<T1, T2, T3> Test<T1, T2, T3> {
    pub fn new<S: Into<String>>(
        name: S,
        command: Vec<T1>,
        clear_env: bool,
        env: Vec<(T2, T3)>,
    ) -> Test<T1, T2, T3> {
        Test {
            name: name.into(),
            command: command,
            clear_env: clear_env,
            env: env,
        }
    }
}

impl<T1: AsRef<OsStr>, T2: AsRef<OsStr>, T3: AsRef<OsStr>> fmt::Display for Test<T1, T2, T3> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &(ref name, ref value) in &self.env[..] {
            write!(
                f,
                "{}={} ",
                escape(name.as_ref().to_string_lossy()),
                escape(value.as_ref().to_string_lossy())
            )?;
        }

        write!(f, "{}", escape(self.command[0].as_ref().to_string_lossy()))?;
        for arg in &self.command[1..] {
            write!(f, " {}", escape(arg.as_ref().to_string_lossy()))?;
        }

        Ok(())
    }
}

impl<T1: AsRef<OsStr>, T2: AsRef<OsStr>, T3: AsRef<OsStr>> Test<T1, T2, T3> {
    fn run_command(&self) -> io::Result<ExitStatus> {
        let mut command = Command::new(&self.command[0]);
        command.args(&self.command[1..]);

        if self.clear_env {
            command.env_clear();
        }

        for &(ref key, ref value) in &self.env {
            command.env(key, value);
        }

        command.status()
    }

    pub fn run(&self) -> bool {
        eprintln_bold!("Running test {} ({})", self.name, self);

        let command_result = self.run_command();

        match command_result {
            Err(error) => {
                eprintln_red!("Test {} failed: {}", self.name, error);
                false
            }
            Ok(status) => if status.success() {
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
            },
        }
    }
}
