#[macro_use] 
extern crate lazy_static;
extern crate chrono;
extern crate hyper;
extern crate hyper_native_tls;
extern crate serde_json;
extern crate select;
extern crate url;
extern crate regex;


mod client;
mod data;
#[cfg(test)]
mod tests;

pub use self::client::SpaceEmailClient;
pub use self::data::*;

