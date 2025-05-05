# 🍝 edoN 🍜

## Embed Node.js within Rust

Embed the fully featured Nodejs runtime into your Rust application!

Features:
- [x] Bindings for `libnode`
- [x] Native Nodejs extensions via napi-rs bindings
- [x] Multi-threading support
- [x] Evaluate arbitrary JavaScript
- [x] Execute arbitrary n-api code
- [ ] Support for async Rust (coming soon)

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
  let nodejs = edon::Nodejs::load("/path/to/libnode.so")?;

  // Nodejs context runs on its own thread
  // you can have multiple and load balance between them
  let ctx = nodejs.spawn_context()?;

  // Execute JavaScript and TypeScript with
  ctx.eval(r#"
    const message: string = "Hello World TypeScript"
    console.log(message)
  "#)?;

  Ok(())
}
```

## Native Extensions

Register a Napi extension and use the napi-rs API to work with the values

```rust
pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  // Register a native module
  nodejs.napi_module_register(
    "example_native_module", 
    |env, mut exports| {
      // Create an object that looks like
      // { meaning: 42 }

      let key = env.create_string("meaning")?;
      let value = env.create_uint32(42)?;
      exports.set_property(key, value)?;

      // Export it from the module
      Ok(exports)
    })?;

  // Start a Nodejs context
  let ctx0 = nodejs.spawn_context()?;

  // Evaluate arbitrary code within the context
  ctx0.eval(r#"
    const native = process._linkedBinding('example_native_module')
    console.log(native)
  "#)?;

  Ok(())
}
```

## Execute Native code in the Nodejs Context

Run native code against a specific Nodejs context. This is essentially `eval` but using
Node's n-api.

```rust
pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  // Start a Nodejs context
  let ctx0 = nodejs.spawn_context()?;

  // Open a native execution context and set a global variable
  ctx0.exec(|env| {
    // Add the following to globalThis:
    // { 
    //    ...globalThis,
    //    meaning: 42 
    //  }
    
    let mut global_this = env.get_global()?;

    let key = env.create_string("meaning")?;
    let value = env.create_uint32(42)?;

    global_this.set_property(key, value)?;

    Ok(())
  })?;

  // Inspect the value set by the native code
  ctx0.eval("console.log(globalThis.meaning)")?; // "42"

  Ok(())
}

```

## Libnode Shared Library

This requires the `libnode` shared library. Currently Node.js don't provide prebuilt binaries so you have to compile `libnode` yourself.

Offering prebuilt binaries with a C FFI is currently under development, however in the meantime you can download `libnode` from here:

[https://github.com/alshdavid/libnode-prebuilt](https://github.com/alshdavid/libnode-prebuilt)

```bash
mkdir -p /opt/libnode

curl -L \
  --url https://github.com/alshdavid/libnode-prebuilt/releases/download/v22.15.0/libnode-linux-amd64.tar.xz \
  | tar -xJvf - -C /opt/libnode

export EDON_LIBNODE_PATH="/opt/libnode/libnode.so"
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