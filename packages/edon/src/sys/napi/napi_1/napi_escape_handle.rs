use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_escape_handle".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  scope: napi_escapable_handle_scope,
  escapee: napi_value,
  result: *mut napi_value,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_escape_handle(
  env: napi_env,
  scope: napi_escapable_handle_scope,
  escapee: napi_value,
  result: *mut napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env, scope, escapee, result,
  )
}
