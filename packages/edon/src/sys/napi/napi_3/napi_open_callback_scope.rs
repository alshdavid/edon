use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_open_callback_scope".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  resource_object: napi_value,
  context: napi_async_context,
  result: *mut napi_callback_scope,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_open_callback_scope(
  env: napi_env,
  resource_object: napi_value,
  context: napi_async_context,
  result: *mut napi_callback_scope,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    resource_object,
    context,
    result,
  )
}
