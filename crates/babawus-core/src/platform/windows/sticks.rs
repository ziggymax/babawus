//! Sticks on Windows: a placeholder that can't connect, so none of its methods can be called.
//! See `platform/windows.rs`.

use std::convert::Infallible;
use std::io;
use std::path::PathBuf;

use super::unsupported;
use crate::format::Target;
use crate::media::Snapshot;
use crate::platform::{EjectError, Ejected, FormatError};
use crate::versions::Format;

pub struct Sticks {
    never: Infallible,
}

impl Sticks {
    pub fn connect() -> io::Result<Self> {
        Err(unsupported())
    }

    pub fn snapshot(&self) -> Snapshot {
        match self.never {}
    }

    pub fn connect_changed(&self, _changed: impl Fn() + 'static) {
        match self.never {}
    }

    pub async fn format(
        &self,
        _id: &str,
        _format: Format,
        _label: &str,
    ) -> Result<PathBuf, FormatError> {
        match self.never {}
    }

    pub async fn format_target(
        &self,
        _target: &Target,
        _format: Format,
        _label: &str,
    ) -> Result<PathBuf, FormatError> {
        match self.never {}
    }

    pub async fn eject(&self, _id: &str) -> Result<Ejected, EjectError> {
        match self.never {}
    }

    pub async fn unmount_all(&self, _mounted: &[String]) -> Result<(), EjectError> {
        match self.never {}
    }
}
