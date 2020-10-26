pub use inline_c_macro::c;

use std::env;
use std::error::Error;
use std::io::prelude::*;
use std::process::Command;
use tempfile::NamedTempFile;

enum Language {
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

pub fn run_c(
    c_program: &str,
    stdout: &mut Vec<u8>,
    stderr: &mut Vec<u8>,
) -> Result<i32, Box<dyn Error>> {
    run(Language::C, c_program, stdout, stderr)
}

pub fn run_cxx(
    cxx_program: &str,
    stdout: &mut Vec<u8>,
    stderr: &mut Vec<u8>,
) -> Result<i32, Box<dyn Error>> {
    run(Language::CXX, cxx_program, stdout, stderr)
}

fn run(
    language: Language,
    program: &str,
    stdout: &mut Vec<u8>,
    stderr: &mut Vec<u8>,
) -> Result<i32, Box<dyn Error>> {
    let mut program_file = NamedTempFile::new()?;
    program_file.write(program.as_bytes())?;

    let object = NamedTempFile::new()?;
    let object_path = object.path();

    let clang_output = Command::new("clang")
        .arg("-x")
        .arg(language.to_string())
        .arg("-o")
        .arg(object_path)
        .arg(program_file.path())
        .current_dir(env::var("CARGO_MANIFEST_DIR")?)
        .output()?;

    if !clang_output.status.success() {
        *stdout = clang_output.stdout;
        *stderr = clang_output.stderr;

        return Ok(clang_output.status.code().unwrap_or(1));
    }

    let object_output = Command::new(object_path).output()?;

    *stdout = object_output.stdout;
    *stderr = object_output.stderr;

    Ok(object_output.status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as inline_c;

    #[test]
    fn test_run_c() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = run_c(
            r#"
#include <stdio.h>

int main() {
  printf("Hello, World!\n");

  return 0;
}
"#,
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(result.unwrap(), 0);
        assert_eq!(String::from_utf8_lossy(&stdout), "Hello, World!\n");
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_run_cxx() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = run_cxx(
            r#"
#include <stdio.h>

int main() {
  printf("Hello, World!\n");

  return 0;
}
"#,
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(result.unwrap(), 0);
        assert_eq!(String::from_utf8_lossy(&stdout), "Hello, World!\n");
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_c_macro() {
        let (result, stdout, stderr) = c! {
            int main() {
                int x = 1;
                int y = 2;

                return x + y;
            }
        };

        assert_eq!(result.unwrap(), 3);
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_c_macro_with_include() {
        let (result, stdout, stderr) = c! {
            #include <stdio.h>

            int main() {
                printf("Hello, World!\n");

                return 0;
            }
        };

        assert_eq!(result.unwrap(), 0);
        assert_eq!(String::from_utf8_lossy(&stdout), "Hello, World!\n");
        assert!(stderr.is_empty());
    }
}
