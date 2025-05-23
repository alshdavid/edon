pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  // Register a native module
  nodejs.napi_module_register("example_native_module", |env, mut exports| {
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
  ctx0.eval(
    r#"
    const native = process._linkedBinding('example_native_module')
    console.log(native)
  "#,
  )?;

  Ok(())
}
