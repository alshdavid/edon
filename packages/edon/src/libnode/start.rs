use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::CString;

use crate::libnode::mark_running;
use crate::libnode::mark_stopped;
use crate::sys;

pub fn start_blocking<Args: AsRef<str>>(argv: &[Args]) -> crate::Result<()> {
  mark_running()?;
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

  unsafe { sys::node_start(c_args.len() as c_int, c_args.as_ptr()) };

  mark_stopped()?;
  Ok(())
}
