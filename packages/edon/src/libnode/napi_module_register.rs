use std::collections::HashMap;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_uint;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;
use std::sync::LazyLock;
use std::sync::RwLock;

use super::super::sys;

type InitFn = unsafe extern "C" fn(sys::napi::napi_env, sys::napi::napi_value) -> sys::napi::napi_value;

static NAPI_MODULE_NAMES: LazyLock<RwLock<HashMap<String, CString>>> =
  LazyLock::new(Default::default);

fn set_napi_module_register_name<S: AsRef<str>>(
  name: S,
) -> bool {
  let mut napi_module_names = NAPI_MODULE_NAMES.write().unwrap();
  let name = name.as_ref().to_string();
  let cname = CString::new(name.clone()).unwrap();
  if napi_module_names.contains_key(&name) {
    return false;
  }
  napi_module_names.insert(name, cname);
  return true;
}

fn get_napi_module_register_name<S: AsRef<str>>(name: S) -> Option<*const c_char> {
  let napi_module_names = NAPI_MODULE_NAMES.read().unwrap();
  let cname = napi_module_names.get(name.as_ref())?;
  Some(cname.as_ptr())
}

pub fn napi_module_register<
  S: AsRef<str>,
  F: 'static
    + Sync
    + Send
    + Fn(sys::napi::napi_env, sys::napi::napi_value) -> sys::napi::napi_value,
>(
  module_name: S,
  register_function: F,
) {
  if !set_napi_module_register_name(&module_name) {
    return;
  }

  let target_fn_leaked: &'static _ = Box::leak(Box::new(register_function));
  let target_fn_closure = libffi::high::Closure2::new(target_fn_leaked);
  let &target_fn_ptr = target_fn_closure.code_ptr();
  let target_fn: InitFn = unsafe { std::mem::transmute(target_fn_ptr) };
  std::mem::forget(target_fn_closure);

  let nm = Box::into_raw(Box::new(sys::napi::napi_module {
    nm_version: 131 as c_int,
    nm_flags: 0 as c_uint,
    nm_filename: get_napi_module_register_name(&module_name).unwrap(),
    nm_register_func: Some(target_fn),
    nm_modname: get_napi_module_register_name(&module_name).unwrap(),
    nm_priv: get_napi_module_register_name(&module_name).unwrap() as *mut c_void,
    reserved: [
      0 as *mut c_void,
      ptr::null_mut(),
      ptr::null_mut(),
      ptr::null_mut(),
    ],
  }));

  unsafe {
    sys::napi::napi_module_register(nm);
  }
}
