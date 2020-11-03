use std::{fs, path::PathBuf, process::Output};

pub struct Assert {
    output: Output,
    object_file: Option<PathBuf>,
}

impl Drop for Assert {
    fn drop(&mut self) {
        if let Some(object_file) = &self.object_file {
            fs::remove_file(object_file).expect("Failed to remove the object file.");
        }
    }
}

impl Assert {
    pub fn new(output: Output, object_file: Option<PathBuf>) -> Self {
        Self {
            output,
            object_file,
        }
    }

    pub fn success(self) -> Self {
        if !self.output.status.success() {
            panic!(
                "Unexpected failure.\ncode={}\nstderr=\n> ```\n>  {}\n> ```\n",
                self.output.status.code().unwrap_or(1),
                String::from_utf8_lossy(&self.output.stderr).replace("\n", "\n> "),
            )
        }

        self
    }

    pub fn failure(self) -> Self {
        if self.output.status.success() {
            panic!("Unexpected success");
        }

        self
    }

    pub fn interrupted(self) -> Self {
        if self.output.status.code().is_some() {
            panic!("Unexpected completion");
        }

        self
    }

    pub fn code(self, expected_code: i32) -> Self {
        let received_code = self
            .output
            .status
            .code()
            .unwrap_or_else(|| panic!("Command interrupted, not code available."));

        assert_eq!(expected_code, received_code, "Codes mismatch");

        self
    }

    pub fn stdout<T>(self, expected_stdout: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        let received_code = self.output.stdout.as_slice();

        assert_eq!(expected_stdout.as_ref(), received_code, "Stdout mismatch");

        self
    }

    pub fn no_stdout(self) -> Self {
        assert!(self.output.stdout.is_empty(), "Stdout is not empty");

        self
    }

    pub fn stderr<T>(self, expected_stderr: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        let received_code = self.output.stderr.as_slice();

        assert_eq!(expected_stderr.as_ref(), received_code, "Stderr mismatch");

        self
    }

    pub fn no_stderr(self) -> Self {
        assert!(self.output.stderr.is_empty(), "Stderr is not empty");

        self
    }
}
