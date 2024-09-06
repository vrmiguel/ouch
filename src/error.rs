//! Error types definitions.
//!
//! All usage errors will pass through the Error enum, a lot of them in the Error::Custom.

use std::{
    borrow::Cow,
    fmt::{self, Display},
};

use crate::{accessible::is_running_in_accessible_mode, utils::colors::*};

/// All errors that can be generated by `ouch`
#[derive(Debug)]
pub enum Error {
    /// Not every IoError, some of them get filtered by `From<io::Error>` into other variants
    IoError { reason: String },
    /// From lzzzz::lz4f::Error
    Lz4Error { reason: String },
    /// Detected from io::Error if .kind() is io::ErrorKind::NotFound
    NotFound { error_title: String },
    /// NEEDS MORE CONTEXT
    AlreadyExists { error_title: String },
    /// From zip::result::ZipError::InvalidArchive
    InvalidZipArchive(&'static str),
    /// Detected from io::Error if .kind() is io::ErrorKind::PermissionDenied
    PermissionDenied { error_title: String },
    /// From zip::result::ZipError::UnsupportedArchive
    UnsupportedZipArchive(&'static str),
    /// TO BE REMOVED
    CompressingRootFolder,
    /// Specialized walkdir's io::Error wrapper with additional information on the error
    WalkdirError { reason: String },
    /// Custom and unique errors are reported in this variant
    Custom { reason: FinalError },
    /// Invalid format passed to `--format`
    InvalidFormat { reason: String },
    /// From sevenz_rust::Error
    SevenzipError(sevenz_rust::Error),
    /// Recognised but unsupported format
    // currently only RAR when built without the `unrar` feature
    UnsupportedFormat { reason: String },
    /// Invalid password provided
    InvalidPassword { reason: String },
    /// UnrarError From unrar::error::UnrarError
    UnrarError { reason: String },
}

/// Alias to std's Result with ouch's Error
pub type Result<T> = std::result::Result<T, Error>;

/// A string either heap-allocated or located in static storage
pub type CowStr = Cow<'static, str>;

/// Pretty final error message for end users, crashing the program after display.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FinalError {
    /// Should be made of just one line, appears after the "\[ERROR\]" part
    title: CowStr,
    /// Shown as a unnumbered list in yellow
    details: Vec<CowStr>,
    /// Shown as green at the end to give hints on how to work around this error, if it's fixable
    hints: Vec<CowStr>,
}

impl Display for FinalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Title
        //
        // When in ACCESSIBLE mode, the square brackets are suppressed
        if is_running_in_accessible_mode() {
            write!(f, "{}ERROR{}: {}", *RED, *RESET, self.title)?;
        } else {
            write!(f, "{}[ERROR]{} {}", *RED, *RESET, self.title)?;
        }

        // Details
        for detail in &self.details {
            write!(f, "\n - {}{}{}", *YELLOW, detail, *RESET)?;
        }

        // Hints
        if !self.hints.is_empty() {
            // Separate by one blank line.
            writeln!(f)?;
            // to reduce redundant output for text-to-speech systems, braille
            // displays and so on, only print "hints" once in ACCESSIBLE mode
            if is_running_in_accessible_mode() {
                write!(f, "\n{}hints:{}", *GREEN, *RESET)?;
                for hint in &self.hints {
                    write!(f, "\n{hint}")?;
                }
            } else {
                for hint in &self.hints {
                    write!(f, "\n{}hint:{} {}", *GREEN, *RESET, hint)?;
                }
            }
        }

        Ok(())
    }
}

impl FinalError {
    /// Only constructor
    #[must_use]
    pub fn with_title(title: impl Into<CowStr>) -> Self {
        Self {
            title: title.into(),
            details: vec![],
            hints: vec![],
        }
    }

    /// Add one detail line, can have multiple
    #[must_use]
    pub fn detail(mut self, detail: impl Into<CowStr>) -> Self {
        self.details.push(detail.into());
        self
    }

    /// Add one hint line, can have multiple
    #[must_use]
    pub fn hint(mut self, hint: impl Into<CowStr>) -> Self {
        self.hints.push(hint.into());
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            Error::WalkdirError { reason } => FinalError::with_title(reason.to_string()),
            Error::NotFound { error_title } => FinalError::with_title(error_title.to_string()).detail("File not found"),
            Error::CompressingRootFolder => {
                FinalError::with_title("It seems you're trying to compress the root folder.")
                    .detail("This is unadvisable since ouch does compressions in-memory.")
                    .hint("Use a more appropriate tool for this, such as rsync.")
            }
            Error::IoError { reason } => FinalError::with_title(reason.to_string()),
            Error::Lz4Error { reason } => FinalError::with_title(reason.to_string()),
            Error::AlreadyExists { error_title } => {
                FinalError::with_title(error_title.to_string()).detail("File already exists")
            }
            Error::InvalidZipArchive(reason) => FinalError::with_title("Invalid zip archive").detail(*reason),
            Error::PermissionDenied { error_title } => {
                FinalError::with_title(error_title.to_string()).detail("Permission denied")
            }
            Error::UnsupportedZipArchive(reason) => FinalError::with_title("Unsupported zip archive").detail(*reason),
            Error::InvalidFormat { reason } => FinalError::with_title("Invalid archive format").detail(reason.clone()),
            Error::Custom { reason } => reason.clone(),
            Error::SevenzipError(reason) => FinalError::with_title("7z error").detail(reason.to_string()),
            Error::UnsupportedFormat { reason } => {
                FinalError::with_title("Recognised but unsupported format").detail(reason.clone())
            }
            Error::InvalidPassword { reason } => FinalError::with_title("Invalid password").detail(reason.clone()),
            Error::UnrarError { reason } => FinalError::with_title("Unrar error").detail(reason.clone()),
        };

        write!(f, "{err}")
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound {
                error_title: err.to_string(),
            },
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied {
                error_title: err.to_string(),
            },
            std::io::ErrorKind::AlreadyExists => Self::AlreadyExists {
                error_title: err.to_string(),
            },
            _other => Self::IoError {
                reason: err.to_string(),
            },
        }
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(err: zip::result::ZipError) -> Self {
        use zip::result::ZipError;
        match err {
            ZipError::Io(io_err) => Self::from(io_err),
            ZipError::InvalidArchive(filename) => Self::InvalidZipArchive(filename),
            ZipError::FileNotFound => Self::Custom {
                reason: FinalError::with_title("Unexpected error in zip archive").detail("File not found"),
            },
            ZipError::UnsupportedArchive(filename) => Self::UnsupportedZipArchive(filename),
        }
    }
}

#[cfg(feature = "unrar")]
impl From<unrar::error::UnrarError> for Error {
    fn from(err: unrar::error::UnrarError) -> Self {
        Self::Custom {
            reason: FinalError::with_title("Unexpected error in rar archive").detail(format!("{:?}", err.code)),
        }
    }
}

impl From<sevenz_rust::Error> for Error {
    fn from(err: sevenz_rust::Error) -> Self {
        Self::SevenzipError(err)
    }
}

impl From<ignore::Error> for Error {
    fn from(err: ignore::Error) -> Self {
        Self::WalkdirError {
            reason: err.to_string(),
        }
    }
}

impl From<FinalError> for Error {
    fn from(err: FinalError) -> Self {
        Self::Custom { reason: err }
    }
}
