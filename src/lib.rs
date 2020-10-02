use inline_c_macro::c;
use std::error::Error;
use std::io::prelude::*;
use std::process::Command;
use tempfile::NamedTempFile;

pub fn run_c(
    c_program: &str,
    stdout: &mut Vec<u8>,
    stderr: &mut Vec<u8>,
) -> Result<i32, Box<dyn Error>> {
    let mut c_program_file = NamedTempFile::new()?;
    c_program_file.write(c_program.as_bytes())?;

    let object = NamedTempFile::new()?;
    let object_path = object.path();

    let clang_output = Command::new("clang")
        .arg("-x")
        .arg("c")
        .arg("-o")
        .arg(object_path)
        .arg(c_program_file.path())
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
    }

    #[test]
    fn test_c_macro() {
        let (result, _stdout, _stderr) = c! {
            int main() {
                int x = 1;
                int y = 2;

                return x + y;
            }
        };

        assert_eq!(result.unwrap(), 3);
    }
}
