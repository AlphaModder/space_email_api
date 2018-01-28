use chrono;
use hyper;

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum SpaceEmailColor {
    Yellow,
    Red,
    Lime,
    Cyan,
    Blue,
    White,
    Pink,
}

impl SpaceEmailColor {
    pub(crate) fn from_css(class: &str) -> SpaceEmailColor {
        match class {
            "msg-red" => SpaceEmailColor::Red,
            "msg-lime" => SpaceEmailColor::Lime,
            "msg-cyan" => SpaceEmailColor::Cyan,
            "msg-blue" => SpaceEmailColor::Blue,
            "msg-white" => SpaceEmailColor::White,
            "msg-pink" => SpaceEmailColor::Pink,
            _ => SpaceEmailColor::Yellow
        }
    }
    
    pub(crate) fn into_id(&self) -> u32 {
        match *self {
            SpaceEmailColor::Yellow => 0,
            SpaceEmailColor::Red => 1,
            SpaceEmailColor::Lime => 2,
            SpaceEmailColor::Cyan => 3,
            SpaceEmailColor::Blue => 4,
            SpaceEmailColor::White => 5,
            SpaceEmailColor::Pink => 6,
        }
    }
    
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct SpaceEmailContents {
    pub subject: String,
    pub sender: String,
    pub body: String,
    pub color: SpaceEmailColor,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct SpaceEmail {
    pub(crate) id: u32,
    pub(crate) share_id: String,
    pub(crate) timestamp: chrono::NaiveDateTime,
    pub(crate) contents: SpaceEmailContents,
}

impl SpaceEmail {

    pub fn id(&self) -> u32 { self.id }

    pub fn timestamp(&self) -> chrono::NaiveDateTime { self.timestamp }

    pub fn contents(&self) -> &SpaceEmailContents { &self.contents }

    pub fn share_id(&self) -> &str { &self.share_id }

    pub fn share_url(&self) -> String { format!("https://space.galaxybuster.net/shv.php?id={}", self.share_id) }

}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
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
    Network(hyper::error::Error),
    MalformedResponse(String),
    InvalidParameter,
    RequiresLogin,
}
