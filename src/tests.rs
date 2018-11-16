extern crate tokio_core;
extern crate native_tls;
extern crate tokio_tls;

use std::sync::Arc;
use std::io;
use std::str::FromStr;

use futures::{future, Future};
use hyper::Uri;
use hyper::client::{Client, Connect, Service, HttpConnector};

use self::tokio_core::reactor::Core;
use self::tokio_core::net::TcpStream;
use self::tokio_tls::TlsStream;
use self::tokio_tls::TlsConnectorExt;
use self::native_tls::TlsConnector;

use super::*;

const TEST_ID: u32 = 139244;
const TEST_SENDER: &str = "Homura Akemi";
const TEST_SUBJECT: &str = "Something to Remember";
const TEST_BODY: &str = "Always, somewhere, someone is fighting for you.";
const TEST_TIMESTAMP: &str = "2014-06-28 16:05";

struct HttpsConnector {
    tls: Arc<TlsConnector>,
    http: HttpConnector,
}

impl Service for HttpsConnector {
    type Request = Uri;
    type Response = TlsStream<TcpStream>;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = io::Error>>;

    fn call(&self, uri: Uri) -> Self::Future {
        if uri.scheme() != Some("https") {
            return future::err(io::Error::new(io::ErrorKind::Other, "only works with https")).boxed()
        }

        let host = match uri.host() {
            Some(s) => s.to_string(),
            None => return future::err(io::Error::new(io::ErrorKind::Other, "missing host")).boxed()
        };

        let tls_cx = self.tls.clone();
        Box::new(self.http.call(uri).and_then(move |tcp| {
            tls_cx.connect_async(&host, tcp).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        }))
    }
}

fn create_client() -> (Core, SpaceEmailClient<HttpsConnector>) {
    let core = Core::new().unwrap();
    let tls_cx: TlsConnector = native_tls::TlsConnector::builder().unwrap().build().unwrap();
    let mut connector = HttpsConnector {
        tls: Arc::new(tls_cx),
        http: HttpConnector::new(2, &core.handle()),
    };
    connector.http.enforce_http(false);
    let client = hyper::Client::configure().connector(connector).build(&core.handle());
    (core, SpaceEmailClient::new(client))
}

#[test]
fn get_email() {
    let (mut core, client) = create_client();
    let email = core.run(client.get_by_id(TEST_ID)).unwrap();
    assert_eq!(email.contents().sender, TEST_SENDER);
    assert_eq!(email.contents().subject, TEST_SUBJECT);
    assert_eq!(email.contents().body, TEST_BODY);
    assert_eq!(email.timestamp().format("%Y-%m-%d %H:%M").to_string(), TEST_TIMESTAMP);
}

#[test]
fn get_random() {
    let (mut core, client) = create_client();
    core.run(client.get_random()).unwrap();
}

#[test]
fn send_email() {
    let (mut core, client) = create_client();
    core.run(client.send(&SpaceEmailContents {
        sender: TEST_SENDER.to_string(),
        subject: TEST_SUBJECT.to_string(),
        body: TEST_BODY.to_string(),
        style: SpaceEmailStyle::Yellow,
    })).unwrap();
}


