pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_default(edon::auto_resolve_libnode()?)?;

  // Evaluate CJS script to set a global variable
  nodejs.eval_blocking("globalThis.meaningOfLife = 42;")?;

  // Evaluate CJS script that inspects the global variable
  nodejs.eval_blocking("console.log(globalThis.meaningOfLife)")?;

  Ok(())
}
