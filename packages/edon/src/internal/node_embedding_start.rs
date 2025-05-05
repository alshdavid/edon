use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::CString;

pub fn start_blocking<Args: AsRef<str>>(argv: &[Args]) -> crate::Result<()> {
  let current_exe = CString::new(std::env::current_exe().unwrap().to_str().unwrap()).unwrap();

  let args = argv
    .iter()
    .map(|arg| CString::new(arg.as_ref()).unwrap())
    .collect::<Vec<CString>>();

  let mut final_args = vec![current_exe];
  final_args.extend(args);

  let c_args = final_args
    .iter()
    .map(|arg| arg.as_ptr())
    .collect::<Vec<*const c_char>>();

  unsafe { libnode_sys::node_embedding_start(c_args.len() as c_int, c_args.as_ptr()) };

  Ok(())
}

pub fn eval_blocking<Code: AsRef<str>>(code: Code) -> crate::Result<()> {
  start_blocking(&[
    "--experimental-strip-types",
    "--disable-warning=ExperimentalWarning",
    "-e",
    code.as_ref(),
  ])?;
  Ok(())
}
