use std::env;
use std::path::PathBuf;
use test::Test;

const CONFIG_FILE_NAME: &'static str = "multitest.toml";


pub fn find_config_file_dir() -> Option<PathBuf> {
    let current_dir = env::current_dir().unwrap();
    let mut current = &*current_dir;

    loop {
        let config_file = current.join(CONFIG_FILE_NAME);
        if config_file.metadata().is_ok() {
            return Some(current.to_path_buf());
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

pub fn load_config() -> Result<Vec<Test<String, String, String>>, ()> {
    // We move to the directory containing the configuration file. This way tests are always executed from this directory.
    match find_config_file_dir() {
        Some(dir) => {
            if env::set_current_dir(dir).is_err() {
                eprintln_red!("Cannot move the directory containing {}", CONFIG_FILE_NAME);
                return Err(());
            }
        }
        None => {
            eprintln_red!("{} not found", CONFIG_FILE_NAME);
            return Err(());
        }
    };

    unimplemented!();
}
