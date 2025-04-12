# 🍝 edoN 🍜

## Embed Node.js within Rust

Embed the fully featured Nodejs runtime into your Rust application!

Features:
- Bindings for `libnode`
- Native Nodejs extensions via standard napi bindings
- Support for worker threads

Todo:
- (help wanted) Build Nodejs statically and link/vendor it into the crate to allow creation of portable single binary outputs
- Deno backend behind `deno_backend` feature
- Bun backend behind `bun_backend` feature
- Quickjs backend behind `quickjs_backend` feature

# Usage

## Simple Example

```rust
pub fn main() -> std::io::Result<()> {
  // Execute JavaScript and TypeScript with
  edon::eval_blocking(r#"
    const message: string = "Hello World TypeScript"
    console.log(message)
    console.log(process._linkedBinding("my_native_extension"))
  "#)?;

  Ok(())
}
```

## Native Extensions

```rust
pub fn main() -> std::io::Result<()> {
  // Inject a native extension into the JavaScript runtime
  // to create/interact with JavaScript

  // Note: This shares the same thread-safe idiosyncrasies as napi
  edon::napi_module_register("my_native_extension", |env, exports| unsafe {
    // Create number
    let mut raw_value = ptr::null_mut();
    edon::sys::napi::napi_create_uint32(env, 42, &mut raw_value);

    // Set number on exports object
    edon::sys::napi::napi_set_named_property(
      env,
      exports.cast(),
      CString::new("hello").unwrap().as_ptr(),
      raw_value,
    );

    // Return exports object
    exports
  });

  // Execute JavaScript and access the native extensions via process._linkedBinding
  edon::eval_blocking(r#"
    console.log('Hello World')
    console.log(process._linkedBinding("my_native_extension")) // { hello: 42 }
  "#)?;

Ok(())
}
```


## Libnode Shared Library

My end goal is that `edon` is installed via crates.io and be used without any additional downloads or fragile configurations. It should be turn-key & batteries included.

I have plans to compile libnode to a statically linked library and vendor it within the crate however I have made a few attempts at building Node.js this way without success. If anyone has experience with building Node.js or has experience vendoring static libraries in Rust crates - PRs, [issues](https://github.com/alshdavid/edon/issues), [discussions](https://github.com/alshdavid/edon/discussions) are welcome!  

However, currently the `libnode.so` / `libnode.dylib` / `libnode.dll` must be downloaded from the [Github releases](https://github.com/alshdavid/edon/releases) and placed next to the executable. Alternatively the path to the shared library can be specified via the `EDON_LIBNODE_PATH` variable.



```rust
fn main() {
  // Currently the libnode.so / libnode.dylib / libnode.dll must be
  // placed next to the executable or the path set in this variable.
  // In the future Will statically compile libnode
  println!("{:?}", std::env::var("EDON_LIBNODE_PATH"))
}
```
