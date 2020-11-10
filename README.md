<h1 align="center">
  <img src="./doc/lilac.jpg" alt="Lilac-breated Roller, by David Clode" width="300px" /><br />
  inline-c
</h1>

Have you ever written a C API for your Rust program? I'm sure you did,
otherwise what are you waiting for? It's fun! Have you ever dreamed
of running your C API directly in your Rust implementation, for
example to unit test it? Nah? Because I did. Bah, I'm probably not the
only one. Right? Please tell me I'm not.

The `inline-c` crate allows you to write C code within Rust directly,
to compile it and to run some assertions. Note that C and Rust are
fully sandboxed, values cannot be exchanged between the two. The
purpose of `inline-c` is to ease the testing of a C API, that is for
example automatically generated with [`cbindgen`].

## Install

Add the following lines to your `Cargo.toml` file:

```toml
[dev-dependencies]
inline-c = "0.1"
```

## Example

### The `assert_c!` and `assert_cxx!` macros

Basic usage of the `assert_c!` (or `assert_cxx!`) macro. In the
following example a simple _Hello, World!_ C program is compiled and
executed. It is then asserted than the exit code and the outputs are
correct. The next example asserts than the C program correctly returns
an error.

```rust
#[test]
fn test_successful() {
    (assert_c! {
        #include <stdio.h>

        int main() {
            printf("Hello, World!\n");

            return 0;
        }
    })
    .success()
    .stdout("Hello, World!\n");
}

#[test]
fn test_badly() {
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
```

### Define environment variables

Now, let's enter the real reason to live of this project. Let's way we
want a C program to link against a specific shared library.

> Note: The `CFLAGS`, `CXXFLAGS`, `CPPFLAGS` and `LDFLAGS` are
> supported environment variables.

Great! We may want to define a value for the `CFLAGS` and the
`LDFLAGS` environment variables. First way is to use the
`#inline_c_rs` C directive with the following syntax:

```
#inline_c_rs <variable_name>: "<variable_value>"
```

Please note the double quotes around the variable value.

Let's see a concrete example. We declare 3 environment variables,
resp. `FOO`, `CFLAGS` and `LDFLAGS`. The C program prints their
corresponding values, and exit accordingly.

```rust
#[test]
fn test_c_macro_with_env_vars_inlined() {
    (assert_c! {
        #inline_c_rs FOO: "bar baz qux"
        #inline_c_rs CFLAGs: "-Ixyz/include -Lzyx/lib"
        #inline_c_rs LDFLAGS: "-lfoo"

        #include <stdio.h>
        #include <stdlib.h>

        int main() {
            const char* foo = getenv("FOO");

            if (NULL == foo) {
                return 1;
            }

            printf("FOO is set to `%s`\n", foo);

            return 0;
        }
    })
    .success()
    .stdout("FOO is set to `bar baz qux`\n");
}
```

This is cool isn't it? But it can be repetitive. What if we can define
environment variables _globally_, for all the C program written in
`assert_c!` or `assert_cxx!`?

It is possible with meta environment variables, with the following syntax:

```
INLINE_C_RS_<variable_name>=<variable_value>
```

Let's see it in action. We set 2 environments variables,
resp. `INLINE_C_RS_FOO`, `INLINE_C_RS_CFLAGS` and
`INLINE_C_RS_LDFLAGS`, that will create `FOO`, `CFLAGS` and `LDFLAGS`
for this C program specifically:

```rust
#[test]
fn test_c_macro_with_env_vars_from_env_vars() {
    set_var("INLINE_C_RS_FOO", "bar baz qux");
    set_var("INLINE_C_RS_CFLAGS", "-Ixyz/include -Lxyz/lib");
    set_var("INLINE_C_RS_LDFLAGS", "-lfoo");

    (assert_c! {
        #include <stdio.h>
        #include <stdlib.h>

        int main() {
            const char* foo = getenv("FOO");

            if (NULL == foo) {
                return 1;
            }

            printf("FOO is set to `%s`\n", foo);

            return 0;
        }
    })
    .success()
    .stdout("FOO is set to `bar baz qux`\n");

    remove_var("INLINE_C_RS_FOO");
    remove_var("INLINE_C_RS_CFLAGS");
    remove_var("INLINE_C_RS_LDFLAGS");
}
```

Note that we have use
[`set_var`](https://doc.rust-lang.org/std/env/fn.set_var.html) and
[`remove_var`](https://doc.rust-lang.org/std/env/fn.remove_var.html)
to set or remove the environment variables. That's for the sake of
simplicity: It is possible to set those variables before running your
tests or anything.

## License

`BSD-3-Clause`, see `LICENSE.md`.

[`cbindgen`]: https://github.com/eqrion/cbindgen/
