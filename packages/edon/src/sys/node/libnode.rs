use std::{path::PathBuf, sync::OnceLock};

#[cfg(unix)]
pub type DynSymbol<T> = libloading::os::unix::Symbol<T>;

#[cfg(windows)]
pub type DynSymbol<T> = libloading::os::windows::Symbol<T>;

#[cfg(unix)]
pub type DynLibrary = libloading::os::unix::Library;

#[cfg(windows)]
pub type DynLibrary = libloading::os::windows::Library;

static LIBNODE: OnceLock<crate::Result<DynLibrary>> = OnceLock::new();

#[cfg(target_os = "linux")]
pub(crate) static LIB_NAME: &str = "libnode.so";

#[cfg(target_os = "macos")]
pub(crate) static LIB_NAME: &str = "libnode.dylib";

#[cfg(target_os = "windows")]
pub(crate) static LIB_NAME: &str = "libnode.dll";

fn find_libnode() -> crate::Result<PathBuf> {
  match std::env::var("EDON_LIBNODE_PATH") {
    Ok(path) => Ok(PathBuf::from(path)),
    Err(_) => {
      let current_exe = std::env::current_exe()?;
      let target1 = current_exe.join(LIB_NAME);
      
      if target1.exists() {
        return Ok(target1)
      }

      if let Some(target2) = current_exe.parent() {
        let target2 = target2.join("lib").join(LIB_NAME);
        if target2.exists() {
          return Ok(target2)
        }
      }

      return Err(crate::Error::LibnodeNotFound);
    }
  }
}

pub fn libnode() -> &'static crate::Result<DynLibrary> {
  LIBNODE.get_or_init(|| {
    let libnode_path = find_libnode()?;
    match unsafe { DynLibrary::new(libnode_path) } {
        Ok(lib) => Ok(lib),
        Err(_) => Err(crate::Error::LibnodeFailedToLoad),
    }
  })
}

pub unsafe fn libnode_sym<T>(symbol: &[u8]) -> crate::Result<DynSymbol<T>> {
  let lib = match libnode() {
    Ok(lib) => lib,
    Err(err) => return Err(crate::Error::from(err)),
  };
  match lib.get(symbol.as_ref()) {
    Ok(sym) => Ok(sym),
    Err(_) => Err(crate::Error::LibnodeSymbolNotFound),
  }
}
