use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_new_instance".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  constructor: napi_value,
  argc: usize,
  argv: *const napi_value,
  result: *mut napi_value,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_new_instance(
  env: napi_env,
  constructor: napi_value,
  argc: usize,
  argv: *const napi_value,
  result: *mut napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    constructor,
    argc,
    argv,
    result,
  )
}
