use super::start_blocking;

pub fn eval_blocking<Code: AsRef<str>>(code: Code) -> crate::Result<()> {
  start_blocking(&["--experimental-strip-types", "--disable-warning=ExperimentalWarning", "-e", code.as_ref()])?;
  Ok(())
}
