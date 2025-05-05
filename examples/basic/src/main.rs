pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  nodejs.napi_module_register("my_test_module", |env, mut exports| {
    let num = env.create_uint32(32)?;
    exports.set_named_property("world", num)?;
    Ok(exports)
  })?;

  nodejs.exec(|env| {
    let mut global_this = env.get_global()?;

    let num = env.create_uint32(32)?;

    global_this.set_named_property("hello", num)?;

    Ok(())
  })?;

  nodejs.eval_script(
    r#"
    setTimeout(() => console.log('done'), 1000)
    console.log(globalThis.hello);
    console.log(process._linkedBinding("my_test_module"));  
  "#,
  )?;

  // let w0 = nodejs.spawn_worker()?;
  // w0.eval(r#"console.log('hello world')"#)?;

  Ok(())
}
