use glob::glob;
use liquid::{self, Context, Renderable, Template};
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use test::Test;
use toml::Value;

const CONFIG_FILE_NAME: &'static str = "multitest.toml";

pub struct TestTemplate {
    pub name: Template,
    pub command: Vec<Template>,
    pub clear_env: bool,
    pub env: Vec<(Template, Template)>,
}

impl TestTemplate {
    fn try_from_test(test: &Test<String, String, String>) -> Result<TestTemplate, ()> {
        let name_template = match liquid::parse(&*test.name, Default::default()) {
            Ok(name_template) => name_template,
            Err(error) => {
                eprintln_red!("error while parsing name template: {}", error);
                return Err(());
            }
        };

        let command_templates = test.command
            .iter()
            .map(|arg| {
                     liquid::parse(&*arg, Default::default()).map_err(|error| {
                    eprintln_red!("error while parsing an arg template: {}", error)
                })
                 })
            .collect::<Result<Vec<_>, ()>>()?;

        let env_templates = test.env
            .iter()
            .map(|&(ref name, ref value)| {
                let name = liquid::parse(name, Default::default())
                    .map_err(|error| {
                                 eprintln_red!("error while parsing an arg template: {}", error)
                             })?;
                let value = liquid::parse(value, Default::default())
                    .map_err(|error| {
                                 eprintln_red!("error while parsing an arg template: {}", error)
                             })?;

                Ok((name, value))
            })
            .collect::<Result<Vec<_>, ()>>()?;

        Ok(TestTemplate {
               name: name_template,
               command: command_templates,
               clear_env: test.clear_env,
               env: env_templates,
           })
    }
}

#[derive(Default)]
pub struct RunConfigResult {
    ignored: u32,
    successes: Vec<String>,
    failures: Vec<String>,
}

impl RunConfigResult {
    pub fn merge(&mut self, other: RunConfigResult) {
        self.ignored += other.ignored;
        self.successes.extend(other.successes);
        self.failures.extend(other.failures);
    }

    pub fn summary(&self) {
        let ignored = self.ignored;
        let successes = &self.successes;
        let failures = &self.failures;

        let total = successes.len() + failures.len();

        if !successes.is_empty() {
            eprintln_green!("Successes ({}/{}):", successes.len(), total);
            for success in successes {
                eprintln_green!("  {}", success);
            }
        }

        if !failures.is_empty() {
            eprintln_red!("Failures ({}/{}):", successes.len(), total);
            for failure in failures {
                eprintln_red!("  {}", failure);
            }
        }

        if ignored > 0 {
            eprintln_bold!("{} tests ignored", ignored);
        }

        if total == 0 {
            eprintln_red!("No tests executed")
        }
    }

    pub fn is_success(&self) -> bool {
        let total = self.successes.len() + self.failures.len();

        self.failures.is_empty() && total > 0
    }
}


#[derive(Debug)]
struct Variable {
    name: String,
    values: Vec<liquid::Value>,
}

impl Variable {
    fn try_from_tuple((key, value): (&String, &Value)) -> Result<Variable, ()> {
        let name = key.clone();

        match value.as_array() {
            Some(values) => {
                let values = values.iter().map(toml_value_to_liquid).collect();

                Ok(Variable {
                       name: name,
                       values: values,
                   })
            }
            None => {
                eprintln_red!("The values of the variables must be arrays");
                Err(())
            }
        }
    }
}

struct ParseResult {
    tests: Vec<Test<String, String, String>>,
    includes: Vec<PathBuf>,
}

fn toml_value_to_liquid(toml_value: &Value) -> liquid::Value {
    match *toml_value {
        Value::String(ref value) => liquid::Value::Str(value.clone()),
        Value::Integer(value) => liquid::Value::Num(value as f32),
        Value::Float(value) => liquid::Value::Num(value as f32),
        Value::Boolean(value) => liquid::Value::Bool(value),
        Value::Datetime(ref value) => liquid::Value::Str(value.to_string()),
        Value::Array(ref value) => {
            liquid::Value::Array(value.iter().map(toml_value_to_liquid).collect())
        }
        Value::Table(ref value) => {
            liquid::Value::Object(value
                                      .iter()
                                      .map(|(key, value)| {
                                               (key.clone(), toml_value_to_liquid(value))
                                           })
                                      .collect())
        }
    }
}

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

fn test_from_toml(test: &Value) -> Result<Test<String, String, String>, ()> {
    let name = match test.get("name").and_then(Value::as_str) {
        Some(name) => name,
        None => {
            eprintln_red!("Error: test without a name");
            return Err(());
        }
    };

    let command = match test.get("command").and_then(Value::as_array) {
        Some(command) => {
            let command: Option<Vec<_>> = command
                .iter()
                .map(Value::as_str)
                .map(|arg| arg.map(|s| s.to_string()))
                .collect();
            match command {
                Some(command) => command,
                None => {
                    eprintln_red!("Error: invalid command for \"{}\"", name);
                    return Err(());
                }
            }
        }
        None => {
            eprintln_red!("Error: test without command");
            return Err(());
        }
    };

    let clear_env = test.get("clear_env")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let env = match test.get("env").and_then(Value::as_array) {
        Some(env) => {
            let env: Result<Vec<_>, ()> = env.iter().map(env_from_table).collect();
            env?
        }
        None => vec![],
    };

    Ok(Test::new(name, command, clear_env, env))
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

fn gen_matrices(test_template: &TestTemplate,
                variables: &[Variable],
                variables_values: &mut HashMap<String, liquid::Value>,
                collected_test: &mut Vec<Test<String, String, String>>)
                -> Result<(), ()> {
    if variables.is_empty() {
        let mut context = Context::with_values(variables_values.clone());
        let name = match test_template.name.render(&mut context) {
            Ok(Some(name)) => name,
            Ok(None) => "".to_string(),
            Err(error) => {
                eprintln_red!("error while rendering name template: {}", error);
                return Err(());
            }
        };

        variables_values.insert("name".to_string(), liquid::Value::Str(name.clone()));

        let command = test_template
            .command
            .iter()
            .map(|arg_template| {
                let mut context = Context::with_values(variables_values.clone());

                match arg_template.render(&mut context) {
                    Ok(Some(name)) => Ok(name),
                    Ok(None) => Ok("".to_string()),
                    Err(error) => {
                        eprintln_red!("error while rendering a name template: {}", error);
                        Err(())
                    }
                }
            })
            .collect::<Result<Vec<_>, ()>>()?;

        let env = test_template
            .env
            .iter()
            .map(|&(ref name_template, ref value_template)| {
                let mut context = Context::with_values(variables_values.clone());
                let name = match name_template.render(&mut context) {
                    Ok(Some(name)) => Ok(name),
                    Ok(None) => Ok("".to_string()),
                    Err(error) => {
                        eprintln_red!("error while rendering an environment variable name \
                                       template: {}",
                                      error);
                        Err(())
                    }
                }?;

                let mut context = Context::with_values(variables_values.clone());
                let value = match value_template.render(&mut context) {
                    Ok(Some(name)) => Ok(name),
                    Ok(None) => Ok("".to_string()),
                    Err(error) => {
                        eprintln_red!("error while rendering an environment variable value \
                                       template: {}",
                                      error);
                        Err(())
                    }
                }?;

                Ok((name, value))
            })
            .collect::<Result<Vec<_>, ()>>()?;

        collected_test.push(Test::new(name, command, test_template.clear_env, env));

        Ok(())
    } else {
        let current_variable = &variables[0];
        let current_variable_name = &current_variable.name;
        for value in &current_variable.values {
            variables_values.insert(current_variable_name.to_string(), value.clone());
            gen_matrices(test_template,
                         &variables[1..],
                         variables_values,
                         collected_test)?;
        }

        Ok(())
    }
}

fn parse_config(config_filename: &Path) -> Result<ParseResult, ()> {
    let mut config_file = match File::open(&*config_filename) {
        Ok(file) => file,
        Err(error) => {
            eprintln_red!("Cannot open {}: {}", config_filename.display(), error);
            return Err(());
        }
    };


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
            let test_template = TestTemplate::try_from_test(&test_from_toml(test)?)?;

            let variables = match test.get("variables").and_then(Value::as_table) {
                Some(table) => {
                    let variables: Result<Vec<_>, ()> =
                        table.into_iter().map(Variable::try_from_tuple).collect();
                    variables?
                }
                None => vec![],
            };

            gen_matrices(&test_template,
                         &variables[..],
                         &mut HashMap::new(),
                         &mut collected_tests)?;
        }
    }

    let mut collected_includes = vec![];

    if let Some(includes) = config_parsed.get("includes").and_then(Value::as_array) {
        for include in includes {
            match Value::as_str(include) {
                Some(include) => {
                    match glob(include) {
                        Ok(paths) => {
                            for path in paths {
                                match path {
                                    Ok(path) => collected_includes.push(path),
                                    Err(err) => {
                                        eprintln_red!("{}", err);
                                        return Err(());
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            eprintln_red!("Invalid include glob pattern: {}", err);
                            return Err(());
                        }
                    }
                }
                None => {
                    eprintln_red!("includes must be strings");
                    return Err(());
                }
            }
        }
    }

    Ok(ParseResult {
           tests: collected_tests,
           includes: collected_includes,
       })
}

pub fn run_config(config_filename: &Path, filter: &Option<Regex>) -> Result<RunConfigResult, ()> {
    let mut result: RunConfigResult = Default::default();
    let current_dir = match env::current_dir() {
        Ok(current_dir) => current_dir,
        Err(err) => {
            eprintln_red!("cannot get the current working directory: {}", err);
            return Err(());
        }
    };

    let ParseResult { tests, includes } = parse_config(config_filename)?;

    // We move to the directory containing the configuration file. This way tests are always
    // executed from this directory.
    let config_dir = config_filename.parent().unwrap();

    if config_dir.to_str() != Some("") {
        if let Err(error) = env::set_current_dir(config_dir) {
            eprintln_red!("Cannot move the directory containing {}: {}",
                          config_filename.display(),
                          error);
            return Err(());
        }
    }

    for include in &includes {
        eprintln_bold!("Including {}", include.display());
        result.merge(run_config(include, filter)?);
    }

    if !includes.is_empty() {
        eprintln_bold!("Going back to {}", config_filename.display());
    }

    for test in tests {
        if let Some(ref regex) = *filter {
            if !regex.is_match(&*test.name) {
                result.ignored += 1;
                eprintln_bold!("Test {} ignored", test.name);
                continue;
            }
        }

        let test_success = test.run();
        if test_success {
            result.successes.push(test.name);
        } else {
            result.failures.push(test.name);
        }
    }


    if let Err(error) = env::set_current_dir(current_dir) {
        eprintln_red!("Cannot move back to the previous working directory: {}",
                      error);
        return Err(());
    }

    Ok(result)
}


pub fn run_config_root(config_filename: Option<&OsStr>,
                       filter: &Option<Regex>)
                       -> Result<RunConfigResult, ()> {
    let config_filename = match config_filename.map(PathBuf::from).or_else(find_config_file) {
        Some(config_filename) => config_filename,
        None => {
            eprintln_red!("{} not found", CONFIG_FILE_NAME);
            return Err(());
        }
    };

    run_config(&config_filename, filter)
}
