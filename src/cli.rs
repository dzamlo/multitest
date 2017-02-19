use clap::{App, Arg};
use regex::Regex;
use std::error::Error;

pub fn build_cli() -> App<'static, 'static> {
    App::new("multitest")
        .about("Runs multiple tests")
        .version(crate_version!())
        .arg(Arg::with_name("config_file")
            .long("config")
            .value_name("CONFIG_FILE")
            .help("Select a configuration file instead of searching for a multitest.toml file"))
        .arg(Arg::with_name("filter")
            .long("filter")
            .value_name("FILTER")
            .validator(|filter| {
                Regex::new(&*filter).map(|_| ()).map_err(|e| e.description().to_string())
            })
            .help("Only run tests that match the filter"))
}
