pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  let wk0 = nodejs.spawn_context()?;
  let wk1 = nodejs.spawn_context()?;
  let wk2 = nodejs.spawn_context()?;
  
  wk0.eval("globalThis.i = 0;")?;
  wk1.eval("globalThis.i = 0;")?;
  wk2.eval("globalThis.i = 0;")?;

  for _ in 0..100 {
    wk0.eval("globalThis.i += 1")?;
    wk1.eval("globalThis.i += 1")?;
    wk2.eval("globalThis.i += 1")?;
  }

  wk0.eval("console.log(globalThis.i)")?;
  wk1.eval("console.log(globalThis.i)")?;
  wk2.eval("console.log(globalThis.i)")?;

  Ok(())
}
