extern crate tokio_core;
use self::tokio_core::Core;

use super::*;

const TEST_ID: u32 = 139244;
const TEST_SENDER: &str = "Homura Akemi";
const TEST_SUBJECT: &str = "Something to Remember";
const TEST_BODY: &str = "Always, somewhere, someone is fighting for you.";
const TEST_TIMESTAMP: &str = "2014-06-28 16:05";

fn create_client() -> (Core, SpaceEmailClient) {
    let mut core = Core::new().unwrap();
    let client = hyper::Client::new(&core.handle());
    SpaceEmailClient::new(client)
}

#[test]
fn get_email() {
    let email = create_client().1.get_id(TEST_ID).unwrap();
    assert_eq!(email.contents().sender, TEST_SENDER);
    assert_eq!(email.contents().subject, TEST_SUBJECT);
    assert_eq!(email.contents().body, TEST_BODY);
    assert_eq!(email.timestamp().format("%Y-%m-%d %H:%M").to_string(), TEST_TIMESTAMP);
}

#[test]
fn get_random() {
    create_client().1.get_random().unwrap();
}

#[test]
fn send_email() {
    create_client().1.send(&SpaceEmailContents {
        sender: TEST_SENDER.to_string(),
        subject: TEST_SUBJECT.to_string(),
        body: TEST_BODY.to_string(),
        color: SpaceEmailColor::Yellow,
    }).unwrap();
}


