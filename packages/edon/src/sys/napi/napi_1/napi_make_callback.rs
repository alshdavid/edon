use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_make_callback".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  async_context: napi_async_context,
  recv: napi_value,
  func: napi_value,
  argc: usize,
  argv: *const napi_value,
  result: *mut napi_value,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_make_callback(
  env: napi_env,
  async_context: napi_async_context,
  recv: napi_value,
  func: napi_value,
  argc: usize,
  argv: *const napi_value,
  result: *mut napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    async_context,
    recv,
    func,
    argc,
    argv,
    result,
  )
}
