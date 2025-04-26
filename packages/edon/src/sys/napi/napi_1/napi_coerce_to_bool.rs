use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_coerce_to_bool".as_bytes();
type SIGNATURE = fn(env: napi_env, value: napi_value, result: *mut napi_value) -> napi_status;
static CACHE: OnceLock<super::super::super::library::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_coerce_to_bool(
  env: napi_env,
  value: napi_value,
  result: *mut napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::library::get_sym(SYMBOL).unwrap())(env, value, result)
}
