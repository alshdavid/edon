pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_default(edon::auto_resolve_libnode()?)?;

  // Start a new Nodejs worker
  let worker = nodejs.spawn_worker_thread()?;

  // Evaluate CJS script to set a global variable
  worker.eval("globalThis.meaningOfLife = 42;")?;

  // Evaluate CJS script that inspects the global variable
  worker.eval("console.log(globalThis.meaningOfLife)")?;

  Ok(())
}
