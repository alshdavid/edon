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

  // Evaluate CJS script that inspects the value set by the native code
  ctx0.eval("console.log(globalThis.meaning)")?; // "42"

  // Evaluate ESM script that demonstrates waiting for tasks to end before continuing
  ctx0.eval_module(
    r#"
    import('node:fs')
      .then(() => console.log(globalThis.meaning));
  "#,
  )?; // "42"

  // Evaluate ESM script that prints out the contents of cwd
  ctx0.eval_module(
    r#"
    import * as fs from 'node:fs'
    import * as fs from 'node:process'

    console.log(fs.readdirSync(process.cwd()))
  "#,
  )?; // "42"

  Ok(())
}
