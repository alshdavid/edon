use std::ffi::c_char;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_throw_error".as_bytes();
type SIGNATURE = fn(env: napi_env, code: *const c_char, msg: *const c_char) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_throw_error(
  env: napi_env,
  code: *const c_char,
  msg: *const c_char,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, code, msg)
}
