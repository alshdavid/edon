pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  nodejs.napi_module_register("my_test_module", |_,_| {
    todo!();
  });

  nodejs.exec(|env| {
    let mut global_this = env.get_global()?;

    let num = env.create_uint32(32).unwrap();

    global_this.set_named_property("hello", num).unwrap();

    Ok(())
  });

  nodejs.eval_script(
    r#"
    console.log(globalThis.hello);
    // process._linkedBinding("my_native_extension");  
  "#,
  )?;

  // let w0 = nodejs.spawn_worker()?;
  // w0.eval(r#"console.log('hello world')"#)?;

  Ok(())
}
