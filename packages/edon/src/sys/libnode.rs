use std::sync::OnceLock;

#[cfg(not(windows))]
pub type DynSymbol<T> = libloading::os::unix::Symbol<T>;

#[cfg(windows)]
pub type DynSymbol<T> = libloading::os::windows::Symbol<T>;

#[cfg(not(windows))]
pub type DynLibrary = libloading::os::unix::Library;

#[cfg(windows)]
pub type DynLibrary = libloading::os::windows::Library;

static LIBNODE: OnceLock<DynLibrary> = OnceLock::new();

pub fn libnode() -> &'static DynLibrary {
  LIBNODE.get_or_init(|| {
    let libnode_path = std::env::var("LIBNODE_PATH").expect("$LIBNODE_PATH not set");
    unsafe { DynLibrary::new(libnode_path).expect("failed to load libnode") }
  })
}

pub fn libnode_sym<T>(symbol: &[u8]) -> DynSymbol<T> {
  unsafe { libnode().get(symbol.as_ref()) }.expect("Unable to find symbol in library")
}
