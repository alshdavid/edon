use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_async_destroy".as_bytes();
type SIGNATURE = fn(env: napi_env, async_context: napi_async_context) -> napi_status;

static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_async_destroy(
  env: napi_env,
  async_context: napi_async_context,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, async_context)
}
