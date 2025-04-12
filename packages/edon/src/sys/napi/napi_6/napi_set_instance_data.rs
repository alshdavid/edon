use std::ffi::c_void;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_set_instance_data".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  data: *mut c_void,
  finalize_cb: napi_finalize,
  finalize_hint: *mut c_void,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_set_instance_data(
  env: napi_env,
  data: *mut c_void,
  finalize_cb: napi_finalize,
  finalize_hint: *mut c_void,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    data,
    finalize_cb,
    finalize_hint,
  )
}
