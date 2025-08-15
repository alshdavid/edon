pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_default(edon::auto_resolve_libnode()?)?;

  // Evaluate CJS script to set a global variable
  nodejs.eval("globalThis.meaningOfLife = 42;")?;

  // Evaluate CJS script that inspects the global variable
  nodejs.eval("console.log(globalThis.meaningOfLife)")?;

  Ok(())
}
