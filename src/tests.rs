use crate::*;

const TEST_ID: u32 = 139244;
const TEST_SENDER: &str = "Homura Akemi";
const TEST_SUBJECT: &str = "Something to Remember";
const TEST_BODY: &str = "Always, somewhere, someone is fighting for you.";
const TEST_TIMESTAMP: &str = "2014-06-28 16:05";

#[tokio::test]
async fn get_email() {
    let client = SpaceEmailClient::new().expect("get_email: failed to create SpaceEmailClient");
    let email = client.get_by_id(TEST_ID).await.expect("get_email: failed to get email by id");
    assert_eq!(email.contents().sender, TEST_SENDER);
    assert_eq!(email.contents().subject, TEST_SUBJECT);
    assert_eq!(email.contents().body, TEST_BODY);
    assert_eq!(email.timestamp().format("%Y-%m-%d %H:%M").to_string(), TEST_TIMESTAMP);
}

#[tokio::test]
async fn get_random() {
    let client = SpaceEmailClient::new().expect("get_random: failed to create SpaceEmailClient");
    client.get_random().await.expect("get_random: failed to get a random email");
}

#[tokio::test]
async fn send_email() {
    let client = SpaceEmailClient::new().expect("send_email: failed to create SpaceEmailClient");
    client.send(&SpaceEmailContents {
        sender: TEST_SENDER.to_string(),
        subject: TEST_SUBJECT.to_string(),
        body: TEST_BODY.to_string(),
        style: SpaceEmailStyle::Yellow,
    }).await.expect("send_email: failed to send email");
}



