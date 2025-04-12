use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_is_dataview".as_bytes();
type SIGNATURE = fn(env: napi_env, value: napi_value, result: *mut bool) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_is_dataview(
  env: napi_env,
  value: napi_value,
  result: *mut bool,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, value, result)
}
