use chrono;
use hyper;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct SpaceEmailContents {
    pub subject: String,
    pub sender: String,
    pub body: String,
    pub color: SpaceEmailColor,
}

#[derive(Debug)]
pub struct SpaceEmail {
    pub id: u32,
    pub timestamp: chrono::NaiveDateTime,
    pub contents: SpaceEmailContents,
    pub share_id: String,
}

#[derive(Debug)]
pub enum SpaceEmailError {
    Network(hyper::error::Error),
    MalformedResponse(String),
    InvalidParameter,
}

