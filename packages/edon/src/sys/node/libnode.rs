use std::{path::PathBuf, sync::OnceLock};

#[cfg(unix)]
pub type DynSymbol<T> = libloading::os::unix::Symbol<T>;

#[cfg(windows)]
pub type DynSymbol<T> = libloading::os::windows::Symbol<T>;

#[cfg(unix)]
pub type DynLibrary = libloading::os::unix::Library;

#[cfg(windows)]
pub type DynLibrary = libloading::os::windows::Library;

static LIBNODE: OnceLock<DynLibrary> = OnceLock::new();

fn find_libnode() -> std::io::Result<PathBuf> {
  match std::env::var("EDON_LIBNODE_PATH") {
    Ok(path) => Ok(PathBuf::from(path)),
    Err(_) => {
      #[cfg(target_os = "linux")]
      let target = std::env::current_exe()?.join("libnode.so");

      #[cfg(target_os = "macos")]
      let target = std::env::current_exe()?.join("libnode.dylib");

      #[cfg(target_os = "windows")]
      let target = std::env::current_exe()?.join("libnode.dll");

      if !target.exists() {
        return Err(std::io::Error::other(""));
      }
      Ok(target)
    }
  }
}

pub fn libnode() -> &'static DynLibrary {
  LIBNODE.get_or_init(|| {
    let libnode_path = find_libnode().expect("NotFound: libnode.so / libnode.dlib / libnode.dll\nPlease place it next to executable or specify it with $EDON_LIBNODE_PATH variable");
    unsafe { DynLibrary::new(libnode_path).expect("failed to load libnode") }
  })
}

pub fn libnode_sym<T>(symbol: &[u8]) -> DynSymbol<T> {
  unsafe { libnode().get(symbol.as_ref()) }.expect("Unable to find symbol in library")
}
