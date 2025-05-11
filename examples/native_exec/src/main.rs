use std::time::Duration;

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

  ctx0.eval_module(r#"await import('node:fs').then(() => console.log(1));"#)?; // "42"
  // ctx0.eval_module(r#"
  //   import * as fs from 'node:fs'
  //   console.log(globalThis.done)
  //   console.log(fs)
  // "#)?; // "42"

  // std::thread::sleep(Duration::from_secs(2));
  Ok(())
}
