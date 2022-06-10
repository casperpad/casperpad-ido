//! Error handling on the casper platform.
use casper_types::ApiError;

/// Errors which can be returned by the library.
///
/// When an `Error` is returned from a smart contract, it is converted to an [`ApiError::User`].
///
/// Where a smart contract consuming this library needs to define further error variants, it can
/// return those via the [`Error::User`] variant or equivalently via the [`ApiError::User`]
/// variant.
///
/// Such a user error should be in the range `[0..(u16::MAX - 4)]` (i.e. [0, 65532]) to avoid
/// conflicting with the other `Error` variants.
#[repr(u16)]
pub enum Error {
    InsufficientBalance = 0,
    InsufficientAllowance = 1,

    // User Error
    PermissionDenied = 41,
    NotExistAuction = 42,
    AlreadyExistAuction = 43,
    SaleNotStarted = 44,
    SaleEnded = 45,
    InvalidTime = 46,
    NotWhiteListed = 47,
    InvalidCSPRPrice = 48,
    AlreadyClaimed = 49,
    TierNotSetted = 50,
    OutOfTier = 51,
    OutOfCapacity = 52,
    NotExistOrder = 53,
    InvalidSchedule = 54,
    InvalidPayToken = 55,

    // Contract Error
    InvalidContext = 90,
    KeyAlreadyExists = 91,
    KeyMismatch = 92,
    Overflow = 93,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}
