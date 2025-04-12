use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_async_init".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  async_resource: napi_value,
  async_resource_name: napi_value,
  result: *mut napi_async_context,
) -> napi_status;

static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_async_init(
  env: napi_env,
  async_resource: napi_value,
  async_resource_name: napi_value,
  result: *mut napi_async_context,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    async_resource,
    async_resource_name,
    result,
  )
}
