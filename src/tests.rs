use super::*;

const TEST_ID: u32 = 139244;
const TEST_SENDER: &str = "Homura Akemi";
const TEST_SUBJECT: &str = "Something to Remember";
const TEST_BODY: &str = "Always, somewhere, someone is fighting for you.";
const TEST_TIMESTAMP: &str = "2014-06-28 16:05";

#[test]
fn get_email() {
    let client = SpaceEmailClient::new();
    let email = client.get_id(TEST_ID).unwrap();
    assert_eq!(email.contents.sender, TEST_SENDER);
    assert_eq!(email.contents.subject, TEST_SUBJECT);
    assert_eq!(email.contents.body, TEST_BODY);
    assert_eq!(email.timestamp.format("%Y-%m-%d %H:%M").to_string(), TEST_TIMESTAMP);
}

#[test]
fn get_random() {
    let client = SpaceEmailClient::new();
    client.get_random().unwrap();
}

#[test]
fn send_email() {
    let client = SpaceEmailClient::new();
    assert!(client.send(&SpaceEmailContents {
        sender: TEST_SENDER.to_string(),
        subject: TEST_SUBJECT.to_string(),
        body: TEST_BODY.to_string(),
        color: SpaceEmailColor::None,
    }).is_ok());
}


