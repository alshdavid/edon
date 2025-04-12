use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_get_uv_event_loop".as_bytes();
type SIGNATURE = fn(env: napi_env, loop_: *mut *mut uv_loop_s) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_get_uv_event_loop(
  env: napi_env,
  loop_: *mut *mut uv_loop_s,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, loop_)
}
