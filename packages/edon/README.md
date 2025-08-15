# 🍝 edoN 🍜

## Embed Node.js within Rust

Embed the fully featured Nodejs runtime into your Rust application!

Features:
- [x] Bindings for `libnode`
- [x] Native Nodejs extensions via napi-rs bindings
- [x] Multi-threading support
- [x] Evaluate arbitrary JavaScript
- [x] Execute arbitrary n-api code
- [ ] Support for async Rust (Experimental, coming soon)

# Use Cases

- Use JavaScript plugins with full support for Nodejs in your Rust application
- Roll your own multi-threaded lambda runtime 
- Roll your own multi-threaded SSR implementation
- Avoid the need to embed your application within an n-api extension

# Usage

## Simple Example

Evaluate JavaScript as a string

```rust
pub fn main() -> std::io::Result<()> {
  // Load the libnode dynamic library
  let nodejs = edon::Nodejs::load_default("/path/to/libnode.so")?;

  // Execute JavaScript with
  nodejs.eval_blocking(r#"
    const message = "Hello World";
    console.log(message);
  "#)?;

  // Execute TypeScript with
  nodejs.eval_typescript_blocking(r#"
    const message: string = "Hello World TypeScript";
    console.log(message);
  "#)?;

  // Execute n-api code with
  nodejs.exec(|env| {
    let mut global_this = env.get_global()?;

    let key = env.create_string("meaningOfLife")?;
    let value = env.create_uint32(42)?;

    global_this.set_property(key, value)?;
    Ok(())
  })?;

  nodejs.eval("console.log(globalThis.meaningOfLife)")?;

  Ok(())
}

```

## Native Extensions

Register a Napi extension and use the napi-rs API to work with the values

```rust
pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_default("/path/to/libnode.so")?;

  // Register a native module
  nodejs.napi_module_register(
    "example_native_module", 
    |env, mut exports| {
      // Create an object that looks like
      // { meaningOfLife: 42 }

      let key = env.create_string("meaningOfLife")?;
      let value = env.create_uint32(42)?;
      exports.set_property(key, value)?;

      // Export it from the module
      Ok(exports)
    })?;

  // Evaluate arbitrary code within the context
  nodejs.eval(r#"
    const native = process._linkedBinding('example_native_module')
    console.log(native) // { meaningOfLife: 42 }
  "#)?;

  Ok(())
}
```

## Execute Native code in the Nodejs Context

Run native code against a specific Nodejs context. This is essentially `eval` but using
Node's n-api.

```rust
pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_default("/path/to/libnode.so")?;

  // Start a Nodejs context
  let ctx0 = nodejs.spawn_worker_thread()?;

  // Open a native execution context and set a global variable
  ctx0.exec(|env| {
    // Add the following to globalThis:
    // { 
    //    ...globalThis,
    //    meaning: 42 
    //  }
    
    let mut global_this = env.get_global()?;

    let key = env.create_string("meaningOfLife")?;
    let value = env.create_uint32(42)?;

    global_this.set_property(key, value)?;

    Ok(())
  })?;

  // Inspect the value set by the native code
  ctx0.eval("console.log(globalThis.meaningOfLife)")?; // "42"

  Ok(())
}
```

## Libnode Shared Library

This requires the `libnode` shared library. Currently Node.js don't provide prebuilt binaries so you have to compile `libnode` yourself.

Offering prebuilt binaries with a C FFI is currently under development, however in the meantime you can download `libnode` from here:

[https://github.com/alshdavid/libnode-prebuilt](https://github.com/alshdavid/libnode-prebuilt)

```bash
mkdir -p $HOME/.local/libnode/24

curl -L \
  --url https://github.com/alshdavid/libnode-prebuilt/releases/download/v24/libnode-linux-amd64.tar.xz \
  | tar -xJvf - -C $HOME/.local/libnode/24

export EDON_LIBNODE_PATH="$HOME/.local/libnode/24/libnode.so"
```

### Distributing `libnode` with your application

I haven't had success in compiling `libnode` to a static library so currently you must include the `libnode` dynamic library alongside your binary.

```
/your-app
  your-app
  libnode.so
```

You can instruct `edon` to automatically find `libnode` by using 

```rust
pub fn main() -> anyhow::Result<()> {
  // Looks through:
  // 
  // $EDON_LIBNODE_PATH
  // <exe_path>/libnode.so
  // <exe_path>/lib/libnode.so
  // <exe_path>/share/libnode.so
  // <exe_path>/../lib/libnode.so
  // <exe_path>/../share/libnode.so
  let nodejs = edon::Nodejs::load_auto()?;
}
```

Or manually specify the path

```rust
pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load("/path/to/libnode.so")?;
}
```

## Credits

Project is inspired by and contains code from:

- [https://github.com/napi-rs/napi-rs](https://github.com/napi-rs/napi-rs)
- [https://github.com/metacall/libnode](https://github.com/metacall/libnode)
