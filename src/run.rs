use crate::assert::Assert;

use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::prelude::*;
use std::process::Command;
use tempfile::NamedTempFile;

pub enum Language {
    C,
    CXX,
}

impl ToString for Language {
    fn to_string(&self) -> String {
        match self {
            Self::C => String::from("c"),
            Self::CXX => String::from("c++"),
        }
    }
}

const ENV_VAR_PREFIX: &'static str = "INLINE_C_RS_";

pub fn run(language: Language, program: &str) -> Result<Assert, Box<dyn Error>> {
    lazy_static! {
        static ref REGEX: Regex =
            Regex::new(r#"#inline_c_rs (?P<variable_name>[^:]+):\s*"(?P<variable_value>[^"]+)"\n"#)
                .unwrap();
    }

    let mut variables = HashMap::new();

    for (variable_name, variable_value) in env::vars().filter_map(|(mut name, value)| {
        if name.starts_with(ENV_VAR_PREFIX) {
            Some((name.split_off(ENV_VAR_PREFIX.len()), value))
        } else {
            None
        }
    }) {
        variables.insert(variable_name, variable_value);
    }

    for captures in REGEX.captures_iter(program) {
        variables.insert(
            String::from(captures["variable_name"].trim()),
            String::from(&captures["variable_value"]),
        );
    }

    let program = &REGEX.replace_all(program, "");

    let mut program_file = NamedTempFile::new()?;
    program_file.write(program.as_bytes())?;

    let object = NamedTempFile::new()?;
    let object_path = object.path();

    let clang_output = Command::new("clang")
        .arg("-x")
        .arg(language.to_string())
        .arg("-O2")
        .arg("-o")
        .arg(object_path)
        .arg(program_file.path())
        .envs(variables.clone())
        .current_dir(env::var("CARGO_MANIFEST_DIR")?)
        .output()?;

    if !clang_output.status.success() {
        return Ok(Assert::new(clang_output));
    }

    Ok(Assert::new(
        Command::new(object_path).envs(variables).output()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_c() {
        run(
            Language::C,
            r#"
#include <stdio.h>

int main() {
    printf("Hello, World!\n");

    return 0;
}
"#,
        )
        .unwrap()
        .success()
        .stdout("Hello, World!\n")
        .no_stderr();
    }

    #[test]
    fn test_run_cxx() {
        run(
            Language::CXX,
            r#"
#include <stdio.h>

int main() {
    printf("Hello, World!\n");

    return 0;
}
"#,
        )
        .unwrap()
        .success()
        .stdout("Hello, World!\n")
        .no_stderr();
    }
}
