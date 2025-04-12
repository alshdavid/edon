use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_open_escapable_handle_scope".as_bytes();
type SIGNATURE = fn(env: napi_env, result: *mut napi_escapable_handle_scope) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_open_escapable_handle_scope(
  env: napi_env,
  result: *mut napi_escapable_handle_scope,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, result)
}
