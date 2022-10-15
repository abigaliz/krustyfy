use crate::dbus_signal::DbusMethod;

///
/// Catchall error type for converting errors from between different libraries as needed
///
/// <3
///
#[derive(Debug)]
pub enum KrustifyError {
    ZbusFdo(zbus::fdo::Error),
    Zvariant(zvariant::Error),
    Send(tokio::sync::mpsc::error::SendError<DbusMethod>),
    Internal { message: String },
}

impl From<zbus::Error> for KrustifyError {
    fn from(err: zbus::Error) -> Self {
        match err {
            zbus::Error::FDO(e) => Self::ZbusFdo(*e),
            zbus::Error::Variant(e) => Self::Zvariant(e),
            _ => Self::Internal {
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
        KrustifyError::Send(err)
    }
}

impl From<KrustifyError> for zbus::fdo::Error {
    fn from(err: KrustifyError) -> Self {
        match err {
            KrustifyError::ZbusFdo(e) => e,
            KrustifyError::Internal { message } => zbus::fdo::Error::Failed(message),
            KrustifyError::Zvariant(e) => zbus::fdo::Error::Failed(e.to_string()),
            KrustifyError::Send(e) => zbus::fdo::Error::Failed(e.to_string()),
        }
    }
}
