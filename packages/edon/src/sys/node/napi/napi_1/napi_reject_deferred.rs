use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_reject_deferred".as_bytes();
type SIGNATURE = fn(env: napi_env, deferred: napi_deferred, rejection: napi_value) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_reject_deferred(
  env: napi_env,
  deferred: napi_deferred,
  rejection: napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, deferred, rejection)
}
