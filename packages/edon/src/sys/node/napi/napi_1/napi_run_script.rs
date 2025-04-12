use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_run_script".as_bytes();
type SIGNATURE = fn(env: napi_env, script: napi_value, result: *mut napi_value) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_run_script(
  env: napi_env,
  script: napi_value,
  result: *mut napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, script, result)
}
