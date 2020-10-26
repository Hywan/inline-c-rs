<h1 align="center">
  <img src="./doc/lilac.jpg" alt="Lilac-breated Roller, by David Clode" /><br />
  inline-c
</h1>

Have you ever written a C API for your Rust program? Have you ever
dreamed of running your C API directly in your Rust implementation,
for example to unit test it? No? Because I did. Bah, I'm probably not
the only one. Right? Please tell me I'm not the only one!

The `inline-c` crate is a really immature project that allows you to
write C code within Rust directly, to compile it and to run some
assertions.

Please, do not use this crate for evil purposes. Stay reasonable. The
main idea is to _test_ a C API, for instance of a Rust program that is
automatically generated with [`cbindgen`].

## Example

```rust
#[cfg(test)]
mod tests {
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
        .stdout("Hello, World!\n")
        .no_stderr();
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
        .code(3)
        .no_stdout()
        .no_stderr();
    }
}
```

## License

`BSD-3-Clause`, see `LICENSE.md`.

[`cbindgen`]: https://github.com/eqrion/cbindgen/
