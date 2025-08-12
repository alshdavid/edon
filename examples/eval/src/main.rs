pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  // Start a new Nodejs context
  let ctx0 = nodejs.spawn_worker_thread()?;

  // Evaluate CJS script to set a global variable
  ctx0.eval("globalThis.meaningOfLife = 42;")?;

  // Evaluate CJS script that inspects the global variable
  ctx0.eval("console.log(globalThis.meaningOfLife)")?;

  // Evaluate ESM script that inspects the global variable
  ctx0.eval_module(
    r#"
    import * as process from 'node:process'
    console.log(globalThis.meaningOfLife)
  "#,
  )?;

  // Evaluate ESM script that demonstrates waiting for tasks to end before continuing
  ctx0.eval_module(
    r#"
    import('node:fs')
      .then(() => console.log(globalThis.meaningOfLife));
  "#,
  )?;

  // Evaluate ESM script that prints out the contents of cwd
  ctx0.eval_module(
    r#"
    import * as fs from 'node:fs'
    import * as process from 'node:process'

    console.log(fs.readdirSync(process.cwd()))
  "#,
  )?;

  Ok(())
}
