# 🍝 edoN 🍜

## Embed Node.js within Rust

Embed the fully featured Nodejs runtime into your Rust application!

Features:
- [x] Bindings for `libnode`
- [x] Native Nodejs extensions via napi-rs bindings
- [x] Multi-threading support
- [x] Evaluate arbitrary JavaScript
- [x] Execute arbitrary n-api code
- [x] Support for async Rust

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
  let ctx = nodejs.spawn_worker_thread()?;

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
  let ctx0 = nodejs.spawn_worker_thread()?;

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
  let ctx0 = nodejs.spawn_worker_thread()?;

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

## Use Async Rust to interact with JsValues 

Use a Rust async runtime that works cooperatively with Nodejs's libuv event loop to work with
JavaScript values in a non-blocking/asynchronous fashion.

```rust
pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  /*
    Register a native extension with the following interface

    interface MyAsyncExtension {
      add_one(a: number): Promise<number>
    }
  */
  nodejs.napi_module_register("my_async_extension", |env, mut exports| {
    // Declare a function called "add_one" on the "module.exports"
    exports.set_named_property("add_one", env.create_function_from_closure("add_one", |ctx| {
      // Get the first argument passed to it. Note that a "JsRc<T>" 
      // prevents the Nodejs GC from freeing the value which allows it
      // to work in async contexts
      let arg0 = ctx.get::<JsRc<JsNumber>>(0)?;
      
      // Spawn a promise that return the summed value
      // Note: This works using a custom async runtime that runs on 
      //       the Nodejs thread cooperatively with libuv.
      //       Make sure you spawn an OS thread or use a runtime like
      //       rayon or tokio _in addition_ to do actual work on
      ctx.env.spawn_local_promise({
        // Clone the env and pass it into the async closure
        let env = ctx.env.clone();
        async move {
          // Do some work asynchronously coordinate via channels.
          async_io::Timer::after(Duration::from_secs(1)).await;
          let value = arg0.get_int32()?;

          // Return the value back to JavaScript
          let js_value = env.create_int32(value + 1)?;
          Ok(js_value)
        }
      })
    })?)?;

    Ok(exports)
  })?;

  // Start a new Nodejs context
  let ctx0 = nodejs.spawn_worker_thread()?;

  // Consume the native module
  ctx0.eval(
    r#"
    const native = process._linkedBinding('my_async_extension');

    (async () => {
      const result = await native.add_one(41)
      console.log(result)                       // "42"
    })();
  "#,
  )?;

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

## Credits

Project is inspired by and contains code from:

- [https://github.com/napi-rs/napi-rs](https://github.com/napi-rs/napi-rs)
- [https://github.com/metacall/libnode](https://github.com/metacall/libnode)
