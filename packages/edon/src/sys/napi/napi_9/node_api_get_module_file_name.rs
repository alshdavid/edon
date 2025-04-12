use std::ffi::c_char;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "node_api_get_module_file_name".as_bytes();
type SIGNATURE = fn(env: napi_env, result: *mut *const c_char) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn node_api_get_module_file_name(
  env: napi_env,
  result: *mut *const c_char,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, result)
}
