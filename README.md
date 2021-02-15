<h1 align="center">
  <img src="./doc/lilac.jpg" alt="Lilac-breated Roller, by David Clode" width="300px" /><br />
  inline-c
</h1>

[![crates.io](https://img.shields.io/crates/v/inline-c)](https://crates.io/crates/inline-c)
[![documentation](https://img.shields.io/badge/doc-inline--c-green)](https://docs.rs/inline-c)

`inline-c` is a small crate that allows a user to write C (including
C++) code inside Rust. Both environments are strictly sandboxed: it is
non-obvious for a value to cross the boundary. The C code is
transformed into a string which is written in a temporary file. This
file is then compiled into an object file, that is finally
executed. It is possible to run assertions about the execution of the
C program.

The primary goal of `inline-c` is to ease the testing of a C API of a
Rust program (generated with
[`cbindgen`](https://github.com/eqrion/cbindgen/) for example). Note
that it's not tied to a Rust program exclusively, it's just its
initial reason to live.

## Install

Add the following lines to your `Cargo.toml` file:

```toml
[dev-dependencies]
inline-c = "0.1"
```

## Documentation

The `assert_c` and `assert_cxx` macros live in the `inline-c-macro`
crate, but are re-exported in this crate for the sake of simplicity.

Being able to write C code directly in Rust offers nice opportunities,
like having C examples inside the Rust documentation that are
executable and thus tested (with `cargo test --doc`). Let's dig into
some examples.

### Basic usage

The following example is super basic: C prints `Hello, World!` on the
standard output, and Rust asserts that.

```rust
use inline_c::assert_c;

fn test_stdout() {
    (assert_c! {
        #include <stdio.h>

        int main() {
            printf("Hello, World!");

            return 0;
        }
    })
    .success()
    .stdout("Hello, World!");
}
```

Or with a C++ program:

```rust
use inline_c::assert_cxx;

fn test_cxx() {
    (assert_cxx! {
        #include <iostream>

        using namespace std;

        int main() {
            cout << "Hello, World!";

            return 0;
        }
    })
    .success()
    .stdout("Hello, World!");
}
```

The `assert_c` and `assert_cxx` macros return a `Result<Assert,
Box<dyn Error>>`. See `Assert` to learn more about the possible
assertions.

The following example tests the returned value:

```rust
use inline_c::assert_c;

fn test_result() {
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

### Environment variables

It is possible to define environment variables for the execution of
the given C program. The syntax is using the special `#inline_c_rs` C
directive with the following syntax:

```c
#inline_c_rs <variable_name>: "<variable_value>"
```

Please note the double quotes around the variable value.

```rust
use inline_c::assert_c;

fn test_environment_variable() {
    (assert_c! {
        #inline_c_rs FOO: "bar baz qux"

        #include <stdio.h>
        #include <stdlib.h>

        int main() {
            const char* foo = getenv("FOO");

            if (NULL == foo) {
                return 1;
            }

            printf("FOO is set to `%s`", foo);

            return 0;
        }
    })
    .success()
    .stdout("FOO is set to `bar baz qux`");
}
```

#### Meta environment variables

Using the `#inline_c_rs` C directive can be repetitive if one needs to
define the same environment variable again and again. That's why meta
environment variables exist. They have the following syntax:

```sh
INLINE_C_RS_<variable_name>=<variable_value>
```

It is usually best to define them in [a `build.rs`
script](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
for example. Let's see it in action with a tiny example:

```rust
use inline_c::assert_c;
use std::env::{set_var, remove_var};

fn test_meta_environment_variable() {
    set_var("INLINE_C_RS_FOO", "bar baz qux");

    (assert_c! {
        #include <stdio.h>
        #include <stdlib.h>

        int main() {
            const char* foo = getenv("FOO");

            if (NULL == foo) {
                return 1;
            }

            printf("FOO is set to `%s`", foo);

            return 0;
        }
    })
    .success()
    .stdout("FOO is set to `bar baz qux`");

    remove_var("INLINE_C_RS_FOO");
}
```

#### `CFLAGS`, `CPPFLAGS`, `CXXFLAGS` and `LDFLAGS`

Some classical `Makefile` variables like `CFLAGS`, `CPPFLAGS`,
`CXXFLAGS` and `LDFLAGS` are understood by `inline-c` and consequently
have a special treatment. Their values are added to the appropriate
compilers when the C code is compiled and linked into an object file.

Pro tip: Let's say we have a Rust crate named `foo`, and it exports a
C API. It is possible to define `CFLAGS` and `LDFLAGS` as follow to
correctly compile and link all the C codes to the Rust `libfoo` shared
object by writing this in a `build.rs` script (it is assumed that
`libfoo` lands in the `target/<profile>/` directory, and that `foo.h`
lands in the root directory):

```rust
use std::{env, path::PathBuf};

fn main() {
    let include_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let mut shared_object_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    shared_object_dir.push("target");
    shared_object_dir.push(env::var("PROFILE").unwrap());
    let shared_object_dir = shared_object_dir.as_path().to_string_lossy();

    // The following options mean:
    //
    // * `-I`, add `include_dir` to include search path,
    // * `-L`, add `shared_object_dir` to library search path,
    // * `-D_DEBUG`, enable debug mode to enable `assert.h`.
    println!(
        "cargo:rustc-env=INLINE_C_RS_CFLAGS=-I{I} -L{L} -D_DEBUG",
        I = include_dir,
        L = shared_object_dir.clone(),
    );

    // Here, we pass the fullpath to the shared object with
    // `LDFLAGS`.
    println!(
        "cargo:rustc-env=INLINE_C_RS_LDFLAGS={shared_object_dir}/{lib}",
        shared_object_dir = shared_object_dir,
        lib = if cfg!(target_os = "windows") {
            "foo.dll".to_string()
        } else if cfg!(target_os = "macos") {
            "libfoo.dylib".to_string()
        } else {
            "libfoo.so".to_string()
        }
    );
}
```

_Et voilà !_ Now run `cargo build --release` (to generate the
shared objects) and then `cargo test --release` to see it in
action.

### Using `inline-c` inside Rust documentation

Since it is now possible to write C code inside Rust, it is
consequently possible to write C examples, that are:

1. Part of the Rust documentation with `cargo doc`, and
2. Tested with all the other Rust examples with `cargo test --doc`.

Yes. Testing C code with `cargo test --doc`. How _fun_ is that? No
trick needed. One can write:

```rust
/// Blah blah blah.
///
/// # Example
///
/// ```rust
/// # use inline_c::assert_c;
/// #
/// # fn main() {
/// #     (assert_c! {
/// #include <stdio.h>
///
/// int main() {
///     printf("Hello, World!");
///
///     return 0;
/// }
/// #    })
/// #    .success()
/// #    .stdout("Hello, World!");
/// # }
/// ```
pub extern "C" fn some_function() {}
```

which will compile down into something like this:

```c
int main() {
    printf("Hello, World!");

    return 0;
}
```

Notice that this example above is actually Rust code, with C code
inside. Only the C code is printed, due to the `#` hack of `rustdoc`,
but this example is a valid Rust example, and is fully tested!

There is one minor caveat though: the highlighting. The Rust set of
rules are applied, rather than the C ruleset. [See this issue on
`rustdoc` to follow the
fix](https://github.com/rust-lang/rust/issues/78917).

### C macros

C macros with the `#define` directive is supported only with Rust
nightly. One can write:

```rust,ignore
use inline_c::assert_c;

fn test_c_macro() {
    (assert_c! {
        #define sum(a, b) ((a) + (b))

        int main() {
            return !(sum(1, 2) == 3);
        }
    })
    .success();
}
```

Note that multi-lines macros don't work! That's because the `\` symbol
is consumed by the Rust lexer. The best workaround is to define the
macro in another `.h` file, and to include it with the `#include`
directive.

## Who is using it?

* [Wasmer](https://github.com/wasmerio/wasmer), the leading
  WebAssembly runtime,
* [Cargo C](https://github.com/lu-zero/cargo-c), to build and install
  C-compatible libraries; it configures `inline-c` for you when using
  `cargo ctest`!
* [Biscuit](https://github.com/CleverCloud/biscuit-rust), an
  authorization token microservices architectures.

## License

`BSD-3-Clause`, see `LICENSE.md`.
