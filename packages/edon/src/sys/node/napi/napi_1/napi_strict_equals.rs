use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_strict_equals".as_bytes();
type SIGNATURE =
  fn(env: napi_env, lhs: napi_value, rhs: napi_value, result: *mut bool) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_strict_equals(
  env: napi_env,
  lhs: napi_value,
  rhs: napi_value,
  result: *mut bool,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, lhs, rhs, result)
}
