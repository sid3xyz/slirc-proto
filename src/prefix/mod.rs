//! IRC message prefix (source) types.

mod serialize;
mod types;

pub use self::types::{is_valid_prefix_str, Prefix, PrefixRef};
