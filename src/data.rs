use chrono;
use hyper;
use std::hash::{Hasher, Hash};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SpaceEmailColor {
    None,
    Red,
    Lime,
    Cyan,
    Blue,
    White,
    Pink,
}

impl SpaceEmailColor {
    
    pub fn from_css(class: &str) -> SpaceEmailColor {
        
        match class {
            "msg-red" => SpaceEmailColor::Red,
            "msg-lime" => SpaceEmailColor::Lime,
            "msg-cyan" => SpaceEmailColor::Cyan,
            "msg-blue" => SpaceEmailColor::Blue,
            "msg-white" => SpaceEmailColor::White,
            "msg-pink" => SpaceEmailColor::Pink,
            _ => SpaceEmailColor::None
        }
    }
    
    pub fn into_id(&self) -> u32 {
        match *self {
            SpaceEmailColor::None => 0,
            SpaceEmailColor::Red => 1,
            SpaceEmailColor::Lime => 2,
            SpaceEmailColor::Cyan => 3,
            SpaceEmailColor::Blue => 4,
            SpaceEmailColor::White => 5,
            SpaceEmailColor::Pink => 6,
        }
    }
    
}

#[derive(Debug, Clone, Eq)]
pub struct SpaceEmailContents {
    pub subject: String,
    pub sender: String,
    pub body: String,
    pub color: SpaceEmailColor,
}

impl PartialEq for SpaceEmailContents {
    fn eq(&self, other: &SpaceEmailContents) -> bool {
        self.subject.eq(&other.subject) && self.sender.eq(&other.sender) && self.body.eq(&other.body)
    }
}

#[derive(Debug, Eq)]
pub struct SpaceEmail {
    id: u32,
    share_id: String,
    timestamp: chrono::NaiveDateTime,
    contents: SpaceEmailContents,
}

impl SpaceEmail {

    pub fn id(&self) -> u32 { self.id }

    pub fn timestamp(&self) -> chrono::NaiveDateTime { self.timestamp }

    pub fn contents(&self) -> &SpaceEmailContents { &self.contents }

    pub fn share_id(&self) -> &str { &self.share_id }

    pub fn share_url(&self) -> String { format!("https://space.galaxybuster.net/shv.php?id={}", self.share_id) }

}

impl Hash for SpaceEmail {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id.hash(state) }
}

impl PartialEq for SpaceEmail {
    fn eq(&self, other: &SpaceEmail) -> bool { self.id == other.id }
}

#[derive(Debug)]
pub enum SpaceEmailError {
    Network(hyper::error::Error),
    MalformedResponse(String),
    InvalidParameter,
}

pub fn create_space_email(id: u32, share_id: &str, timestamp: chrono::NaiveDateTime, contents: SpaceEmailContents) -> SpaceEmail {
    SpaceEmail { id: id, share_id: share_id.to_string(), timestamp: timestamp, contents: contents }
}
