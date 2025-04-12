use std::path::PathBuf;
use tar::Archive;
use xz::read::XzDecoder;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const VENDORED_BYTES: &[u8] = include_bytes!("./libnode-v23.11.0-mac-arm64.tar.xz");

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub const VENDORED_BYTES: &[u8] = &[];

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub const VENDORED_BYTES: &[u8] = &[];

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub const VENDORED_BYTES: &[u8] = include_bytes!("./libnode-v23.11.0-linux-x64.tar.xz");

#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
pub const VENDORED_BYTES: &[u8] = &[];

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const VENDORED_BYTES: &[u8] = &[];

pub fn vendored() -> PathBuf {
    let home = homedir::my_home().unwrap().unwrap();
    let vendor_path = home.join(".config").join("edon").join("vendor");

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let target = vendor_path.join("libnode-v23.11.0-mac-arm64").join("libnode.dylib");

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let target = vendor_path.join("libnode-v23.11.0-mac-x64").join("libnode.dylib");

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let target = vendor_path.join("libnode-v23.11.0-linux-arm64").join("libnode.so");

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let target = vendor_path.join("libnode-v23.11.0-linux-x64").join("libnode.so");

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let target = vendor_path.join("libnode-v23.11.0-windows-x64").join("libnode.dll");

    if !target.exists() {
        println!("extracting");
        let tar = XzDecoder::new(VENDORED_BYTES);
        let mut archive = Archive::new(tar);
        archive.unpack(&vendor_path).unwrap();
    }

    target
}