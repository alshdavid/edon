use edon::Env;

pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  nodejs.env(|env| unsafe {
    let env = Env::from_raw(env);

    let mut global_this = env.get_global().unwrap();

    let num = env.create_uint32(32).unwrap();

    global_this.set_named_property("hello", num).unwrap();
  });

  nodejs.eval(
    r#"
    console.log(globalThis.hello);
    // process._linkedBinding("my_native_extension");  
  "#,
  )?;

  // let w0 = nodejs.spawn_worker()?;
  // w0.eval(r#"console.log('hello world')"#)?;

  Ok(())
}
