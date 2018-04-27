#[macro_use] 
extern crate lazy_static;
extern crate chrono;
extern crate hyper;
extern crate futures;
extern crate serde_json;
extern crate select;
extern crate url;
extern crate regex;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

mod client;
mod data;
#[cfg(test)]
mod tests;

pub use self::client::SpaceEmailClient;
pub use self::data::{SpaceEmail, SpaceEmailContents, SpaceEmailError, SpaceEmailStyle};

