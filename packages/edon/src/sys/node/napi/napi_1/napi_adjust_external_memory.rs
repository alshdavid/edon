use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_adjust_external_memory".as_bytes();
type SIGNATURE = fn(env: napi_env, change_in_bytes: i64, adjusted_value: *mut i64) -> napi_status;

static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_adjust_external_memory(
  env: napi_env,
  change_in_bytes: i64,
  adjusted_value: *mut i64,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    change_in_bytes,
    adjusted_value,
  )
}
