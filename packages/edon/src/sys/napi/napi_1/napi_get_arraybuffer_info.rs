use std::ffi::c_void;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_get_arraybuffer_info".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  arraybuffer: napi_value,
  data: *mut *mut c_void,
  byte_length: *mut usize,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_get_arraybuffer_info(
  env: napi_env,
  arraybuffer: napi_value,
  data: *mut *mut c_void,
  byte_length: *mut usize,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    arraybuffer,
    data,
    byte_length,
  )
}
