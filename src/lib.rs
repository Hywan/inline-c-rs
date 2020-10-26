mod assert;

pub use crate::assert::Assert;
pub use inline_c_macro::{assert_c, assert_cxx};

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

pub fn run_c(c_program: &str) -> Result<Assert, Box<dyn Error>> {
    run(Language::C, c_program)
}

pub fn run_cxx(cxx_program: &str) -> Result<Assert, Box<dyn Error>> {
    run(Language::CXX, cxx_program)
}

fn run(language: Language, program: &str) -> Result<Assert, Box<dyn Error>> {
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
        .current_dir(env::var("CARGO_MANIFEST_DIR")?)
        .output()?;

    if !clang_output.status.success() {
        return Ok(Assert::new(clang_output));
    }

    Ok(Assert::new(Command::new(object_path).output()?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as inline_c;

    #[test]
    fn test_run_c() {
        run_c(
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
        run_cxx(
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
    fn test_c_macro() {
        (assert_c! {
            int main() {
                int x = 1;
                int y = 2;

                return x + y;
            }
        })
        .failure()
        .code(3)
        .no_stdout()
        .no_stderr();
    }

    #[test]
    fn test_c_macro_with_include() {
        (assert_c! {
            #include <stdio.h>

            int main() {
                printf("Hello, World!\n");

                return 0;
            }
        })
        .success()
        .stdout("Hello, World!\n")
        .no_stderr();
    }
}
