//! Local install support for the game.
//!
//! The install step reads the folder the player points at, copies the
//! archives it needs into the player's own D2RUST folder, converts them to
//! native assets, and then bakes animation frames from those assets. All of
//! it happens on the player's machine; nothing is downloaded and no game
//! file is shipped with D2RUST.

pub mod ctable;
pub mod convert;
pub mod fsutil;
pub mod kind;
pub mod kinds;
pub mod names;
pub mod report;
pub mod verify;

pub use convert::{convert, ConvertError, Options, RunSummary};
pub use ctable::TableSummary;
pub use kind::{Failure, Kind, NativeData, Written};
pub use verify::{verify, verify_full, VerifyOptions, VerifyOutcome};

/// The archives an install must contain, in load order.
pub const REQUIRED_ARCHIVES: [&str; 6] = [
    "d2data.mpq",
    "d2exp.mpq",
    "d2char.mpq",
    "d2sfx.mpq",
    "d2speech.mpq",
    "patch_d2.mpq",
];
