use std::ffi::c_void;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_add_finalizer".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  js_object: napi_value,
  native_object: *mut c_void,
  finalize_cb: napi_finalize,
  finalize_hint: *mut c_void,
  result: *mut napi_ref,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_add_finalizer(
  env: napi_env,
  js_object: napi_value,
  native_object: *mut c_void,
  finalize_cb: napi_finalize,
  finalize_hint: *mut c_void,
  result: *mut napi_ref,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    js_object,
    native_object,
    finalize_cb,
    finalize_hint,
    result,
  )
}
