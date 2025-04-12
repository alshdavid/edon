use std::ffi::c_void;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_create_external_arraybuffer".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  external_data: *mut c_void,
  byte_length: usize,
  finalize_cb: napi_finalize,
  finalize_hint: *mut c_void,
  result: *mut napi_value,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_create_external_arraybuffer(
  env: napi_env,
  external_data: *mut c_void,
  byte_length: usize,
  finalize_cb: napi_finalize,
  finalize_hint: *mut c_void,
  result: *mut napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    external_data,
    byte_length,
    finalize_cb,
    finalize_hint,
    result,
  )
}
