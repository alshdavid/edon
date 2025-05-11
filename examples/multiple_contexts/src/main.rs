pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  // Start a pool of contexts
  // Each context runs on its own thread
  let wk0 = nodejs.spawn_context()?;
  let wk1 = nodejs.spawn_context()?;
  let wk2 = nodejs.spawn_context()?;

  // Declare some global variables
  wk0.eval("globalThis.i = 0;")?;
  wk1.eval("globalThis.i = 0;")?;
  wk2.eval("globalThis.i = ;")?;

  // Do work on global variables
  for _ in 0..100 {
    wk0.eval("globalThis.i += 1")?;
    wk1.eval("globalThis.i += 1")?;
    wk2.eval("globalThis.i += 1")?;
  }

  // Inspect values
  wk0.eval("console.log(globalThis.i)")?; // "100"
  wk1.eval("console.log(globalThis.i)")?; // "100"
  wk2.eval("console.log(globalThis.i)")?; // "100"

  Ok(())
}
