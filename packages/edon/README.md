# 🍝 edoN 🍜

## Embed Node.js within Rust

Embed the fully featured Nodejs runtime into your Rust application!

Features:
- Bindings for `libnode`
- Native Nodejs extensions via standard napi bindings
- Support for worker threads

# Usage

## Simple Example

```rust
pub fn main() -> std::io::Result<()> {
  // Load the libnode.so/dylib/dll
  let nodejs = edon::Nodejs::load("/path/to/libnode.so")?;

  // Execute JavaScript and TypeScript with
  nodejs.eval_blocking(r#"
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
  // Load the libnode.so/dylib/dll
  let nodejs = edon::Nodejs::load("/path/to/libnode.so")?;

  // Inject a native extension into the JavaScript runtime
  // to create/interact with JavaScript
  // Note: This shares the same thread-safe idiosyncrasies as napi
  nodejs.napi_module_register("my_native_extension", |env, exports| unsafe {
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
  nodejs.eval_blocking(r#"
    console.log('Hello World')
    console.log(process._linkedBinding("my_native_extension")) // { hello: 42 }
  "#)?;

  Ok(())
}
```


## Libnode Shared Library

My end goal is that `edon` is installed via crates.io and be used without any additional downloads or fragile configurations. It should be turn-key & batteries included.

I have plans to compile libnode to a statically linked library however I have made a few attempts at building Node.js this way without success. If anyone has experience with building Node.js or has experience vendoring static libraries in Rust crates - PRs, [issues](https://github.com/alshdavid/edon/issues), [discussions](https://github.com/alshdavid/edon/discussions) are welcome!  

However, currently the `libnode.so` / `libnode.dylib` / `libnode.dll` must be downloaded from the [Github releases](https://github.com/alshdavid/edon/releases) and placed next to the executable. Alternatively the path to the shared library can be specified via the `EDON_LIBNODE_PATH` variable.

```rust
mkdir -p "$HOME/.config/edon/lib"
curl -L --progress-bar --url "https://github.com/alshdavid/edon/releases/download/v23.11.0-beta.1/libnode-v23.11.0-linux-x64.tar.xz"  | tar -xJzf - -C "$HOME/.config/edon/lib"
ls -l "$HOME/.config/edon/lib/libnode-v23.11.0-linux-x64"

export EDON_LIBNODE_PATH="$HOME/.config/edon/lib/libnode-v23.11.0-linux-x64/libnode.so"
```

```rust
fn main() {
  // Currently the libnode.so / libnode.dylib / libnode.dll must be
  // placed next to the executable or the path set in this variable.
  // In the future Will statically compile libnode
  println!("{:?}", std::env::var("EDON_LIBNODE_PATH"))
}
```
