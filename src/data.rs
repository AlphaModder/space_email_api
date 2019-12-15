use chrono;
use reqwest;
use std::error::Error;
use std::fmt;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SpaceEmailStyle {
    Yellow,
    Red,
    Lime,
    Cyan,
    Blue,
    White,
    Pink,
    Admin,
}

impl SpaceEmailStyle {
    pub(crate) fn from_css(class: &str) -> SpaceEmailStyle {
        match class {
            "msg-red" => SpaceEmailStyle::Red,
            "msg-lime" => SpaceEmailStyle::Lime,
            "msg-cyan" => SpaceEmailStyle::Cyan,
            "msg-blue" => SpaceEmailStyle::Blue,
            "msg-white" => SpaceEmailStyle::White,
            "msg-pink" => SpaceEmailStyle::Pink,
            "admin" => SpaceEmailStyle::Admin,
            _ => SpaceEmailStyle::Yellow
        }
    }
    
    pub(crate) fn into_id(&self) -> Option<u32> {
        match *self {
            SpaceEmailStyle::Yellow => Some(0),
            SpaceEmailStyle::Red => Some(1),
            SpaceEmailStyle::Lime => Some(2),
            SpaceEmailStyle::Cyan => Some(3),
            SpaceEmailStyle::Blue => Some(4),
            SpaceEmailStyle::White => Some(5),
            SpaceEmailStyle::Pink => Some(6),
            SpaceEmailStyle::Admin => None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct EmailId { pub(crate) id: u32, pub(crate) style: SpaceEmailStyle }

impl From<u32> for EmailId {
    fn from(id: u32) -> EmailId {
        EmailId { id, style: SpaceEmailStyle::Yellow }
    }
}

impl Into<u32> for EmailId {
    fn into(self) -> u32 { self.id }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct SpaceEmailContents {
    pub subject: String,
    pub sender: String,
    pub body: String,
    pub style: SpaceEmailStyle,
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct SpaceEmail {
    pub(crate) id: u32,
    pub(crate) share_id: String,
    pub(crate) timestamp: chrono::NaiveDateTime,
    pub(crate) contents: SpaceEmailContents,
}

impl SpaceEmail {
    pub fn id(&self) -> EmailId { EmailId { id: self.id, style: self.contents.style } }

    pub fn timestamp(&self) -> chrono::NaiveDateTime { self.timestamp }

    pub fn contents(&self) -> &SpaceEmailContents { &self.contents }

    pub fn share_id(&self) -> &str { &self.share_id }

    pub fn share_url(&self) -> String { format!("https://space.galaxybuster.net/shv.php?id={}", self.share_id) }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SpaceEmailRange {
    Today,
    Week,
    Month,
    All
}

impl SpaceEmailRange {
    pub(crate) fn into_id(&self) -> u32 {
        match *self {
            SpaceEmailRange::Today => 0,
            SpaceEmailRange::Week => 1,
            SpaceEmailRange::Month => 2,
            SpaceEmailRange::All => 3,
        }
    }
}

#[derive(Debug)]
pub enum SpaceEmailError {
    Network(reqwest::Error),
    MalformedResponse(String),
    InvalidParameter,
    RequiresLogin,
}

impl From<reqwest::Error> for SpaceEmailError {
    fn from(error: reqwest::Error) -> SpaceEmailError {
        SpaceEmailError::Network(error)
    }
}

impl fmt::Display for SpaceEmailError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SpaceEmailError::Network(ref e) => e.fmt(f),
            SpaceEmailError::MalformedResponse(ref s) => write!(f, "Recieved malformed response: {}", s),
            SpaceEmailError::InvalidParameter => write!(f, "Invalid parameter."),
            SpaceEmailError::RequiresLogin => write!(f, "Operation requires the client to be logged in to an account.")
        }
    }
}

impl Error for SpaceEmailError {
    fn description(&self) -> &str {
        match *self {
            SpaceEmailError::Network(ref e) => e.description(),
            SpaceEmailError::MalformedResponse(_) => "Recieved malformed response",
            SpaceEmailError::InvalidParameter => "Invalid parameter",
            SpaceEmailError::RequiresLogin => "Operation requires a logged-in client"
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            SpaceEmailError::Network(ref e) => Some(e),
            _ => None,
        }
    }
}