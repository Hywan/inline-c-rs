mod run;

pub use crate::run::{run, Language};
pub use inline_c_macro::{assert_c, assert_cxx};
pub mod predicates {
    pub use predicates::prelude::*;
}

#[cfg(test)]
mod tests {
    use super::predicates::*;
    use super::*;
    use crate as inline_c;
    use std::env::{remove_var, set_var};

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
        .code(3);
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
        .stdout(predicate::eq("Hello, World!\n").normalize());
    }

    #[test]
    fn test_c_macro_with_env_vars_inlined() {
        (assert_c! {
            // Those are env variables.
            #inline_c_rs FOO: "bar baz qux"
            #inline_c_rs HELLO: "World!"

            #include <stdio.h>

            #ifdef _WIN32
            #include <cstdlib>
            #elif
            #include <stdlib.h>
            #endif


            int main() {
                const char* foo = getenv("FOO");
                const char* hello = getenv("HELLO");

                if (NULL == foo || NULL == hello) {
                    return 1;
                }

                printf("FOO is set to `%s`\n", foo);
                printf("HELLO is set to `%s`\n", hello);

                return 0;
            }
        })
        .success()
        .stdout(
            predicate::eq(
                "FOO is set to `bar baz qux`\n\
                HELLO is set to `World!`\n",
            )
            .normalize(),
        );
    }

    #[test]
    fn test_c_macro_with_env_vars_from_env_vars() {
        // Define env vars through env vars.
        set_var("INLINE_C_RS_FOO", "bar baz qux");
        set_var("INLINE_C_RS_HELLO", "World!");

        (assert_c! {
            #include <stdio.h>
            #include <stdlib.h>

            int main() {
                const char* foo = getenv("FOO");
                const char* hello = getenv("HELLO");

                if (NULL == foo || NULL == hello) {
                    return 1;
                }

                printf("FOO is set to `%s`\n", foo);
                printf("HELLO is set to `%s`\n", hello);

                return 0;
            }
        })
        .success()
        .stdout(
            predicate::eq(
                "FOO is set to `bar baz qux`\n\
                HELLO is set to `World!`\n",
            )
            .normalize(),
        );

        remove_var("INLINE_C_RS_FOO");
        remove_var("INLINE_C_RS_HELLO");
    }
}
