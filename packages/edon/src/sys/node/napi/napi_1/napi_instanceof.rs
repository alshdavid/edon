use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_instanceof".as_bytes();
type SIGNATURE =
  fn(env: napi_env, object: napi_value, constructor: napi_value, result: *mut bool) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_instanceof(
  env: napi_env,
  object: napi_value,
  constructor: napi_value,
  result: *mut bool,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    object,
    constructor,
    result,
  )
}
