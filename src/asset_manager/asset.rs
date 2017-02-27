use engine::Context;

use std::fmt::{Debug, Display, Error as FormatError, Formatter};
use std::marker::Sized;
use std::io::{Error as IoError, ErrorKind};

/// The asset type.
/// Every asset (Mesh, Texture, ...) has to implement
/// this type in order to be compatible with the `AssetLoader`.
///
/// ### The concept
///
/// An asset is loaded in two steps:
///
/// 1) Load asset data (`AssetData`)
/// 2) Turn asset data into asset (`from_data`)
///
/// # Examples
///
/// Here is an examle implementation of an
/// asset:
///
/// ```
/// use amethyst::asset_manager::Asset;
/// use amethyst::engine::Context;
///
/// struct Table {
///     num_rows: u32,
///     num_columns: u32,
///     data: Box<[i32]>,
/// }
///
/// struct WhitespaceTable;
///
/// impl Asset for Table {
///     type Data = Self;
///     type Error = (); // Should be some InconsistentSizeError (if data.len() != num_rows * num_columns)
///
///     fn from_data(data: Self, _: &mut Context) -> Result<Self, ()> {
///         Ok(data) // Ommittedhere: Check for size
///     }
/// }
///
/// impl AssetFormat for WhitespaceTable {
///     fn file_extension() -> &'static str {
///         "wst"
///     }
/// }
/// ```
pub trait Asset: Sized {
    /// The data type, an intermediate format.
    /// This may also be `Self` if this asset does
    /// not depend on `Context`.
    type Data;
    /// The error type that may be returned if
    /// `from_data` fails.
    type Error: Debug;

    /// Create the asset from the data and the context (used to create buffers for the gpu).
    fn from_data(data: Self::Data, context: &mut Context) -> Result<Self, Self::Error>;
}

/// Specifies an asset format. Note that
/// the asset format is not the same as the type.
/// There may be multiple formats for the same
/// asset. A format should also implement the
/// `Import` trait for the target `Asset`s data
/// type.
pub trait AssetFormat {
    /// Return the typical file extension a file
    // with this format has, without the preceding `"."`.
    fn file_extension() -> &'static str;
}

/// An asset store may be a ".zip" file, a server,
/// a custom binary format conataining levels, or just
/// a directory.
pub trait AssetStore {
    /// Read an asset from a given name and format and
    /// return the bytes.
    fn read_asset<F: AssetFormat>(&self,
                                  name: &str,
                                  format: F)
                                  -> Result<Box<[u8]>, AssetStoreError>;
}

/// Error raised if an asset could not be loaded from
/// the asset store.
#[derive(Debug)]
pub enum AssetStoreError {
    /// This asset does not exist in this asset
    /// store. Note that you must not add a file
    /// extension to `name` when accessing an asset.
    /// The file extension comes with the format
    /// parameter.
    NoSuchAsset,
    /// You do not have enough permissions to read
    /// this resource.
    PermissionDenied,
    /// There was a timeout when requesting to read this
    /// resource.
    Timeout,
    /// The asset store you tried to read from is not
    /// available. This may be the case if you tried
    /// to read from some server but it is offline.
    NotAvailable,
    /// Some error which does not match any of the above.
    Other(String),
}

impl From<IoError> for AssetStoreError {
    fn from(e: IoError) -> Self {
        match e.kind() {
            ErrorKind::NotFound => AssetStoreError::NoSuchAsset,
            ErrorKind::PermissionDenied => AssetStoreError::PermissionDenied,
            ErrorKind::ConnectionRefused |
            ErrorKind::ConnectionAborted |
            ErrorKind::ConnectionReset |
            ErrorKind::NotConnected |
            ErrorKind::AddrInUse |
            ErrorKind::AddrNotAvailable |
            ErrorKind::BrokenPipe => AssetStoreError::NotAvailable,
            ErrorKind::TimedOut => AssetStoreError::Timeout,
            x => AssetStoreError::Other(format!("Other: {:?}", x)),
        }
    }
}

impl Display for AssetStoreError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        match self {
            &AssetStoreError::NoSuchAsset => write!(f, "No such asset"),
            &AssetStoreError::PermissionDenied => {
                write!(f, "You do not have enough permissions to access this asset")
            }
            &AssetStoreError::Timeout => {
                write!(f, "A timeout occured when trying to read the asset")
            }
            &AssetStoreError::NotAvailable => write!(f, "The asset storage could not be reached"),
            &AssetStoreError::Other(ref x) => write!(f, "Othere error: {}", x),
        }
    }
}
