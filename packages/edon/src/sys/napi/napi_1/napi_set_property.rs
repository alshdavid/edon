use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_set_property".as_bytes();
type SIGNATURE =
  fn(env: napi_env, object: napi_value, key: napi_value, value: napi_value) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_set_property(
  env: napi_env,
  object: napi_value,
  key: napi_value,
  value: napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, object, key, value)
}
