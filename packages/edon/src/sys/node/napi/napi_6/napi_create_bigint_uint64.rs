use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_create_bigint_uint64".as_bytes();
type SIGNATURE = fn(env: napi_env, value: u64, result: *mut napi_value) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_create_bigint_uint64(
  env: napi_env,
  value: u64,
  result: *mut napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL).unwrap())(env, value, result)
}
