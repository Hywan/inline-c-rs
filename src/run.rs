use crate::assert::Assert;

use lazy_static::lazy_static;
use regex::Regex;
use std::{
    borrow::Cow, collections::HashMap, env, error::Error, ffi::OsString, io::prelude::*,
    path::PathBuf, process::Command,
};

pub enum Language {
    C,
    Cxx,
}

impl ToString for Language {
    fn to_string(&self) -> String {
        match self {
            Self::C => String::from("c"),
            Self::Cxx => String::from("cpp"),
        }
    }
}

pub fn run(language: Language, program: &str) -> Result<Assert, Box<dyn Error>> {
    let (program, variables) = collect_environment_variables(program);

    let mut program_file = tempfile::Builder::new()
        .suffix(&format!(".{}", language.to_string()))
        .tempfile()?;
    program_file.write(program.as_bytes())?;

    #[cfg(target_os = "windows")]
    {
        let file = program_file.as_file();
        let mut permissions = file.metadata()?.permissions();
        dbg!(&permissions);
        dbg!(permissions.readonly());
        permissions.set_readonly(false);
        file.set_permissions(permissions)?;
    }

    let host = target_lexicon::HOST.to_string();
    let target = &host;

    let input_file = program_file.path();
    let output_temp = tempfile::Builder::new().tempfile()?;
    let (_, output_path) = output_temp.keep()?;

    let mut build = cc::Build::new();
    let mut build = build
        .cargo_metadata(false)
        .warnings(true)
        .extra_warnings(true)
        .warnings_into_errors(true)
        .debug(false)
        .host(&host)
        .target(target)
        .opt_level(2);

    if let Language::Cxx = language {
        build = build.cpp(true);
    }

    // Usually, `cc-rs` is used to produce libraries. In our case, we
    // want to produce an (executable) object file. The following code
    // is kind of a hack around `cc-rs`. It avoids the addition of the
    // `-c` argument on the compiler, and manually adds other
    // arguments.

    let compiler = build.try_get_compiler()?;
    let mut command = compiler.to_command();

    command_add_compiler_flags(&mut command, &variables);

    {
        let msvc = target.contains("msvc");
        let clang = compiler.is_like_clang();
        command_add_output_file(&mut command, &output_path, msvc, clang);
    }

    command.arg(input_file);

    let clang_output = command.envs(variables.clone()).output()?;

    if !clang_output.status.success() {
        return Ok(Assert::new(format!("{:?}", command), clang_output, None));
    }

    let mut command = Command::new(output_path.clone());
    command.envs(variables);

    Ok(Assert::new(
        format!("{:?}", command),
        command.output()?,
        Some(output_path),
    ))
}

fn collect_environment_variables<'p>(program: &'p str) -> (Cow<'p, str>, HashMap<String, String>) {
    const ENV_VAR_PREFIX: &'static str = "INLINE_C_RS_";

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
            captures["variable_name"].trim().to_string(),
            captures["variable_value"].to_string(),
        );
    }

    let program = REGEX.replace_all(program, "");

    (program, variables)
}

// This is copy-pasted and edited from `cc-rs`.
fn command_add_output_file(command: &mut Command, output_path: &PathBuf, msvc: bool, clang: bool) {
    if msvc && !clang {
        let mut string = OsString::from("-Fo");
        string.push(output_path);
        command.arg(string);
    } else {
        command.arg("-o").arg(output_path);
    }
}

fn command_add_compiler_flags(command: &mut Command, variables: &HashMap<String, String>) {
    let get_env_flags = |env_name: &str| -> Vec<String> {
        variables
            .get(env_name)
            .map(|e| e.to_string())
            .ok_or_else(|| env::var(env_name))
            .unwrap_or(String::new())
            .split_ascii_whitespace()
            .map(|slice| slice.to_string())
            .collect()
    };

    command.args(get_env_flags("CFLAGS"));
    command.args(get_env_flags("CPPFLAGS"));
    command.args(get_env_flags("CXXFLAGS"));

    for linker_argument in get_env_flags("LDFLAGS") {
        command.arg(format!("-Wl,{}", linker_argument));
    }
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
            Language::Cxx,
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
