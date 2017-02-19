use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use test::Test;
use toml::Value;

const CONFIG_FILE_NAME: &'static str = "multitest.toml";


pub fn find_config_file() -> Option<PathBuf> {
    let current_dir = env::current_dir().unwrap();
    let mut current = &*current_dir;

    loop {
        let config_file = current.join(CONFIG_FILE_NAME);
        if config_file.metadata().is_ok() {
            return Some(config_file.to_path_buf());
        }

        match current.parent() {
            Some(parent) => {
                current = parent;
            }
            None => {
                break;
            }
        }
    }

    None
}

fn env_from_table(table: &Value) -> Result<(String, String), ()> {
    let name = table.get("name").and_then(Value::as_str);
    let value = table.get("value").and_then(Value::as_str);

    match (name, value) {
        (Some(name), Some(value)) => Ok((name.to_string(), value.to_string())),
        (Some(name), None) => {
            eprintln_red!("Error: environment variable \"{}\" without a value", name);
            Err(())
        }
        (None, Some(value)) => {
            eprintln_red!("Error: environment variable with value \"{}\" without a name",
                          value);
            Err(())
        }
        (None, None) => {
            eprintln_red!("Error: environment variable with neither a name or a value");
            Err(())
        }
    }
}

pub fn load_config(config_filename: Option<&OsStr>)
                   -> Result<Vec<Test<String, String, String>>, ()> {

    let config_filename = match config_filename.map(PathBuf::from).or_else(find_config_file) {
        Some(config_filename) => config_filename,
        None => {
            eprintln_red!("{} not found", CONFIG_FILE_NAME);
            return Err(());
        }
    };

    let mut config_file = match File::open(&*config_filename) {
        Ok(file) => file,
        Err(error) => {
            eprintln_red!("Cannot open {}: {}", config_filename.display(), error);
            return Err(());
        }
    };

    // We move to the directory containing the configuration file. This way tests are always executed from this directory.
    let config_dir = config_filename.parent().unwrap();

    if config_dir.to_str() != Some("") {
        if let Err(error) = env::set_current_dir(config_dir) {
            eprintln_red!("Cannot move the directory containing {}: {}",
                          config_filename.display(),
                          error);
            return Err(());
        }
    }


    let mut config_text = String::new();

    if let Err(error) = config_file.read_to_string(&mut config_text) {
        eprintln_red!("Error while reading {}: {}", CONFIG_FILE_NAME, error);
        return Err(());
    }

    let config_parsed = match config_text.parse::<Value>() {
        Ok(config) => config,
        Err(error) => {
            eprintln_red!("Error while parsing {}: {}", CONFIG_FILE_NAME, error);
            return Err(());
        }
    };


    let mut collected_tests = vec![];

    if let Some(tests) = config_parsed.get("tests").and_then(Value::as_array) {
        for test in tests {
            let name = match test.get("name").and_then(Value::as_str) {
                Some(name) => name,
                None => {
                    eprintln_red!("Error: test without a name");
                    return Err(());
                }
            };

            let args = match test.get("args").and_then(Value::as_array) {
                Some(args) => {
                    let args: Option<Vec<_>> = args.iter()
                        .map(Value::as_str)
                        .map(|arg| arg.map(|s| s.to_string()))
                        .collect();
                    match args {
                        Some(args) => args,
                        None => {
                            eprintln_red!("Error: invalid args for \"{}\"", name);
                            return Err(());
                        }
                    }
                }
                None => {
                    eprintln_red!("Error: test without args");
                    return Err(());
                }
            };

            let env = match test.get("env").and_then(Value::as_array) {
                Some(env) => {
                    let env: Result<Vec<_>, ()> = env.iter().map(env_from_table).collect();
                    env?
                }
                None => vec![],
            };


            collected_tests.push(Test::new(name, args, env))
        }

    }

    Ok(collected_tests)
}
