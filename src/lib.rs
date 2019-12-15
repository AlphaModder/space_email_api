#![recursion_limit="512"]

mod client;
mod data;

#[cfg(test)]
mod tests;

pub use self::client::SpaceEmailClient;
pub use self::data::{SpaceEmail, SpaceEmailContents, SpaceEmailError, SpaceEmailStyle};

