use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_call_function".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  recv: napi_value,
  func: napi_value,
  argc: usize,
  argv: *const napi_value,
  result: *mut napi_value,
) -> napi_status;

static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_call_function(
  env: napi_env,
  recv: napi_value,
  func: napi_value,
  argc: usize,
  argv: *const napi_value,
  result: *mut napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env, recv, func, argc, argv, result,
  )
}
