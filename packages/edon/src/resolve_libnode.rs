use std::path::PathBuf;

use libnode_sys::constants::LIB_NAME;

use crate::internal::PathExt;

/// Look for libnode.so / libnode.dylib / libnode.dll from
///
/// * $EDON_LIBNODE_PATH
/// * $EDON_LIBNODE_PATH/libnode.so
/// * <exe_path>/libnode.so
/// * <exe_path>/lib/libnode.so
/// * <exe_path>/share/libnode.so
/// * <exe_path>/../lib/libnode.so
/// * <exe_path>/../share/libnode.so
pub fn auto_resolve_libnode() -> crate::Result<PathBuf> {
  if let Ok(path) = std::env::var("EDON_LIBNODE_PATH") {
    let path = PathBuf::from(&path);

    if path.is_file() {
      return Ok(path);
    }

    let alt = path.join(LIB_NAME);
    if std::fs::exists(&alt)? {
      return Ok(alt);
    }

    Err(crate::Error::generic("Invalid EDON_LIBNODE_PATH"))
  } else {
    let dirname = std::env::current_exe()?.try_parent()?;

    let paths = vec![
      dirname.join(LIB_NAME),
      dirname.join("lib").join(LIB_NAME),
      dirname.join("share").join(LIB_NAME),
      dirname.join("..").join("lib").join(LIB_NAME),
      dirname.join("..").join("share").join(LIB_NAME),
    ];

    for path in paths {
      if std::fs::exists(&path)? {
        return Ok(PathBuf::from(&path));
      }
    }

    Err(crate::Error::LibnodeNotFound)
  }
}
