use std::ffi::CStr;

use libnode_sys;
use crate::napi::Error;
use crate::napi::Status;

#[derive(Debug, Clone, Copy)]
pub struct NodeVersion {
  pub major: u32,
  pub minor: u32,
  pub patch: u32,
  pub release: &'static str,
}

impl TryFrom<libnode_sys::napi_node_version> for NodeVersion {
  type Error = Error;

  fn try_from(value: libnode_sys::napi_node_version) -> Result<NodeVersion, Error> {
    Ok(NodeVersion {
      major: value.major,
      minor: value.minor,
      patch: value.patch,
      release: unsafe {
        CStr::from_ptr(value.release)
          .to_str()
          .map_err(|_| Error::new(Status::StringExpected, "Invalid release name".to_owned()))?
      },
    })
  }
}
