#[cfg(target_os = "linux")]
pub(crate) static LIB_NAME: &str = "libnode.so";

#[cfg(target_os = "macos")]
pub(crate) static LIB_NAME: &str = "libnode.dylib";

#[cfg(target_os = "windows")]
pub(crate) static LIB_NAME: &str = "libnode.dll";
