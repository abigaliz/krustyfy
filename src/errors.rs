use std::{ffi::NulError, sync::PoisonError};

use crate::dbus_signal::{DbusMethod, DbusSignal};

///
/// Catchall error type for converting errors from between different libraries as needed
///
/// <3
///
#[derive(Debug)]
pub enum KrustifyError {
    ZbusFdo(zbus::fdo::Error),
    Zvariant(zvariant::Error),
    DbusMethodSend(tokio::sync::mpsc::error::SendError<DbusMethod>),
    DbusSignalSend(tokio::sync::mpsc::error::SendError<DbusSignal>),
    CStr(NulError),
    FindChild(qt_core::FindChildError),
    Other { message: String },
}

impl From<zbus::Error> for KrustifyError {
    fn from(err: zbus::Error) -> Self {
        match err {
            zbus::Error::FDO(e) => Self::ZbusFdo(*e),
            zbus::Error::Variant(e) => Self::Zvariant(e),
            _ => Self::Other {
                message: err.to_string(),
            },
        }
    }
}

impl From<zvariant::Error> for KrustifyError {
    fn from(err: zvariant::Error) -> Self {
        Self::Zvariant(err)
    }
}

impl From<tokio::sync::mpsc::error::SendError<DbusMethod>> for KrustifyError {
    fn from(err: tokio::sync::mpsc::error::SendError<DbusMethod>) -> Self {
        KrustifyError::DbusMethodSend(err)
    }
}

impl From<NulError> for KrustifyError {
    fn from(err: NulError) -> Self {
        KrustifyError::CStr(err)
    }
}

impl From<qt_core::FindChildError> for KrustifyError {
    fn from(err: qt_core::FindChildError) -> Self {
        KrustifyError::FindChild(err)
    }
}

impl<T> From<PoisonError<std::sync::MutexGuard<'_, T>>> for KrustifyError {
    fn from(err: PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
        Self::Other {
            message: err.to_string(),
        }
    }
}

impl From<tokio::sync::mpsc::error::SendError<DbusSignal>> for KrustifyError {
    fn from(err: tokio::sync::mpsc::error::SendError<DbusSignal>) -> Self {
        Self::DbusSignalSend(err)
    }
}

impl From<KrustifyError> for zbus::fdo::Error {
    fn from(err: KrustifyError) -> Self {
        match err {
            KrustifyError::ZbusFdo(e) => e,
            KrustifyError::Other { message } => zbus::fdo::Error::Failed(message),
            KrustifyError::Zvariant(e) => zbus::fdo::Error::Failed(e.to_string()),
            KrustifyError::DbusMethodSend(e) => zbus::fdo::Error::Failed(e.to_string()),
            KrustifyError::CStr(e) => zbus::fdo::Error::Failed(e.to_string()),
            KrustifyError::FindChild(e) => zbus::fdo::Error::Failed(e.to_string()),
            KrustifyError::DbusSignalSend(e) => zbus::fdo::Error::Failed(e.to_string()),
        }
    }
}
