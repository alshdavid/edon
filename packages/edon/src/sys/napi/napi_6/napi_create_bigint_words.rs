use std::ffi::c_int;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_create_bigint_words".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  sign_bit: c_int,
  word_count: usize,
  words: *const u64,
  result: *mut napi_value,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_create_bigint_words(
  env: napi_env,
  sign_bit: c_int,
  word_count: usize,
  words: *const u64,
  result: *mut napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env, sign_bit, word_count, words, result,
  )
}
