pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_default(edon::auto_resolve_libnode()?)?;

  // Start a pool of contexts
  // Each context runs on its own thread
  let wk0 = nodejs.spawn_worker_thread()?;
  let wk1 = nodejs.spawn_worker_thread()?;
  let wk2 = nodejs.spawn_worker_thread()?;

  // Declare some global variables
  wk0.eval_blocking("globalThis.i = 0;")?;
  wk1.eval_blocking("globalThis.i = 0;")?;
  wk2.eval_blocking("globalThis.i = 0;")?;

  // Do work on global variables
  for _ in 0..100 {
    wk0.eval_blocking("globalThis.i += 1")?;
    wk1.eval_blocking("globalThis.i += 1")?;
    wk2.eval_blocking("globalThis.i += 1")?;
  }

  // Inspect values
  wk0.eval_blocking("console.log(globalThis.i)")?; // "100"
  wk1.eval_blocking("console.log(globalThis.i)")?; // "100"
  wk2.eval_blocking("console.log(globalThis.i)")?; // "100"

  Ok(())
}
