pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_default(edon::auto_resolve_libnode()?)?;

  // Start a Nodejs context
  let worker = nodejs.spawn_worker_thread()?;

  // Open a native execution context and set a global variable
  worker.exec_blocking(|env| {
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

  // Evaluate script that inspects the value set by the native code
  worker.eval_blocking("console.log(globalThis.meaning)")?; // "42"

  // Evaluate script that demonstrates waiting for tasks to end before continuing
  worker.eval_blocking(
    r#"
    ;(async () => {
      console.log(globalThis.meaning)
    })();
  "#,
  )?;

  worker.eval_blocking(
    r#"
    ;(async () => {
      const fs = await import('node:fs')
      const process = await import('node:process')

      console.log(fs.readdirSync(process.cwd()))
    })();
  "#,
  )?;

  Ok(())
}
