use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_cancel_async_work".as_bytes();
type SIGNATURE = fn(env: napi_env, work: napi_async_work) -> napi_status;

static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_cancel_async_work(
  env: napi_env,
  work: napi_async_work,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, work)
}
