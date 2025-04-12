use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_set_element".as_bytes();
type SIGNATURE =
  fn(env: napi_env, object: napi_value, index: u32, value: napi_value) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_set_element(
  env: napi_env,
  object: napi_value,
  index: u32,
  value: napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, object, index, value)
}
