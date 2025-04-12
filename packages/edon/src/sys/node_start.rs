use std::ffi::c_char;
use std::ffi::c_int;
use std::sync::OnceLock;

type SIGNATURE = fn(argc: c_int, argv: *const *const c_char);
static CACHE: OnceLock<super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn node_start(
  argc: c_int,
  argv: *const *const c_char,
) {
  CACHE.get_or_init(|| super::libnode::libnode_sym(b"node_start"))(argc, argv)
}
