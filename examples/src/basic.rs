use edon::NodejsOptions;

pub fn main() -> anyhow::Result<()> {
  // Load the libnode dynamic library
  let nodejs = edon::Nodejs::load(NodejsOptions {
    libnode_path: edon::auto_resolve_libnode()?,
    // Suppress TypeScript warning
    disable_warnings: vec!["ExperimentalWarning".to_string()],
    ..Default::default()
  })?;

  // Execute JavaScript with
  nodejs.eval_blocking(
    r#"
    const message = "Hello World"
    console.log(message)
  "#,
  )?;

  // Execute TypeScript with
  nodejs.eval_typescript_blocking(
    r#"
    const message: string = "Hello World TypeScript"
    console.log(message)
  "#,
  )?;

  // Execute n-api code with
  nodejs.exec_blocking(|env| {
    let mut global_this = env.get_global()?;

    let key = env.create_string("meaningOfLife")?;
    let value = env.create_uint32(42)?;

    global_this.set_property(key, value)?;
    Ok(())
  })?;

  nodejs.eval_blocking("console.log(globalThis.meaningOfLife)")?;

  Ok(())
}
