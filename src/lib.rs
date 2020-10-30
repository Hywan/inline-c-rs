mod assert;
mod run;

pub use crate::assert::Assert;
pub use crate::run::{run, Language};
pub use inline_c_macro::{assert_c, assert_cxx};

#[cfg(test)]
mod tests {
    use super::*;
    use crate as inline_c;

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

    #[test]
    fn test_c_macro_with_inline_c_rs() {
        (assert_c! {
            #inline_c_rs LDFLAGS: "-lfoo"
            #inline_c_rs FOO: "bar baz qux"
            #include <stdio.h>
            #include <stdlib.h>

            int main() {
                const char* foo = getenv("FOO");
                const char* ldflags = getenv("LDFLAGS");

                if (NULL == foo || NULL == ldflags) {
                    return 1;
                }

                printf("FOO is set to `%s`\n", foo);
                printf("LDFLAGS is set to `%s`\n", ldflags);

                return 0;
            }
        })
        .success()
        .stdout(
            "FOO is set to `bar baz qux`\n\
             LDFLAGS is set to `-lfoo`\n",
        )
        .no_stderr();
    }
}
