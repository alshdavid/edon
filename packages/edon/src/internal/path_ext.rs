#![allow(unused)]

use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub trait OsStringExt {
  fn try_to_string(self) -> std::io::Result<String>;
}

impl OsStringExt for OsString {
  fn try_to_string(self) -> std::io::Result<String> {
    match self.into_string() {
      Ok(name) => Ok(name),
      Err(_) => Err(std::io::Error::other(
        "Unable to convert OsString to String",
      )),
    }
  }
}

impl OsStringExt for &OsStr {
  fn try_to_string(self) -> std::io::Result<String> {
    match self.to_str() {
      Some(name) => Ok(name.to_string()),
      None => Err(std::io::Error::other(
        "Unable to convert OsString to String",
      )),
    }
  }
}

impl OsStringExt for Option<&OsStr> {
  fn try_to_string(self) -> std::io::Result<String> {
    match self {
      Some(name) => Ok(name.try_to_string()?),
      None => Err(std::io::Error::other(
        "Unable to convert OsString to String",
      )),
    }
  }
}

pub trait PathExt {
  fn try_parent(&self) -> std::io::Result<PathBuf>;
  fn try_file_name(&self) -> std::io::Result<String>;
  fn try_file_stem(&self) -> std::io::Result<String>;
  fn try_to_string(&self) -> std::io::Result<String>;
}

impl PathExt for PathBuf {
  fn try_parent(&self) -> std::io::Result<PathBuf> {
    match self.parent() {
      Some(path) => Ok(path.to_path_buf()),
      None => Err(std::io::Error::other("Unable to find parent")),
    }
  }

  fn try_file_name(&self) -> std::io::Result<String> {
    match self.file_name() {
      Some(v) => Ok(v.try_to_string()?),
      None => Err(std::io::Error::other("Cannot get file name")),
    }
  }

  fn try_file_stem(&self) -> std::io::Result<String> {
    match self.file_stem() {
      Some(v) => Ok(v.try_to_string()?),
      None => Err(std::io::Error::other("Cannot get file stem")),
    }
  }

  fn try_to_string(&self) -> std::io::Result<String> {
    match self.to_str() {
      Some(v) => Ok(v.to_string()),
      None => Err(std::io::Error::other("Cannot convert Path to string")),
    }
  }
}

impl PathExt for Path {
  fn try_parent(&self) -> std::io::Result<PathBuf> {
    match self.parent() {
      Some(path) => Ok(path.to_path_buf()),
      None => Err(std::io::Error::other("Unable to find parent")),
    }
  }

  fn try_file_name(&self) -> std::io::Result<String> {
    match self.file_name() {
      Some(v) => Ok(v.try_to_string()?),
      None => Err(std::io::Error::other("Cannot get file name")),
    }
  }

  fn try_file_stem(&self) -> std::io::Result<String> {
    match self.file_stem() {
      Some(v) => Ok(v.try_to_string()?),
      None => Err(std::io::Error::other("Cannot get file stem")),
    }
  }

  fn try_to_string(&self) -> std::io::Result<String> {
    match self.to_str() {
      Some(v) => Ok(v.to_string()),
      None => Err(std::io::Error::other("Cannot convert Path to string")),
    }
  }
}

pub fn find_ancestor_file<P: AsRef<Path>, S: AsRef<Path>>(
  start_dir: P,
  file_name: S,
) -> std::io::Result<Vec<PathBuf>> {
  let file_name = file_name.as_ref();
  let mut found = vec![];
  let mut current = start_dir.as_ref().to_path_buf();

  loop {
    let possible = current.join(file_name);

    if fs::exists(&possible)? {
      found.push(possible)
    }

    let Some(next) = current.parent() else {
      break;
    };

    current = next.to_path_buf();
  }

  Ok(found)
}
