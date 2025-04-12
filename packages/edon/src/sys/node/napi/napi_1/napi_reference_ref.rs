use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_reference_ref".as_bytes();
type SIGNATURE = fn(env: napi_env, ref_: napi_ref, result: *mut u32) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_reference_ref(
  env: napi_env,
  ref_: napi_ref,
  result: *mut u32,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, ref_, result)
}
