// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

mod casper_ido;
pub mod constants;
mod data;
pub mod enums;
mod error;
mod event;
mod libs;
pub mod structs;

pub use casper_ido::CasperIdo;
pub use data::Auctions;
pub use data::{auction_purse, create_auction_purse};
pub use error::Error;
