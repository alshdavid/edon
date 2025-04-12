use std::ffi::c_void;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_get_cb_info".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  cbinfo: napi_callback_info,
  argc: *mut usize,
  argv: *mut napi_value,
  this_arg: *mut napi_value,
  data: *mut *mut c_void,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_get_cb_info(
  env: napi_env,
  cbinfo: napi_callback_info,
  argc: *mut usize,
  argv: *mut napi_value,
  this_arg: *mut napi_value,
  data: *mut *mut c_void,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env, cbinfo, argc, argv, this_arg, data,
  )
}
