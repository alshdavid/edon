pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  nodejs.napi_module_register("my_test_module", |env, mut exports| {
    println!("hello world");
    let num = env.create_uint32(32)?;
    exports.set_named_property("world", num)?;
    Ok(exports)
  })?;

  nodejs.exec(|env| {
    let mut global_this = env.get_global()?;

    let num = env.create_uint32(42)?;
    global_this.set_named_property("hello", num)?;

    Ok(())
  })?;

  nodejs.eval_module(
    r#"
    import * as fs from 'node:fs';

    console.log(fs)
    console.log('Done sync' as string)
    console.log(importNative('my_test_module'))
    console.log(globalThis.hello)

    await new Promise(res => setTimeout(() => {
      console.log('Done async')
      res()
    }, 1000))
  "#,
  )?;

  Ok(())
}
