use super::data::*;
use hyper;
use chrono;
use select::document::Document;
use select::predicate::{Class, Attr};
use std::io::Read;
use hyper::Client;
use std::cell::RefCell;
use data::create_space_email;

const GET_ENDPOINT: &'static str = "https://space.galaxybuster.net/lib/get.php";

const VIEW_ENDPOINT: &'static str = "https://space.galaxybuster.net/lib/view.php"; 

const SEND_ENDPOINT: &'static str = "https://space.galaxybuster.net/lib/send.php";

const STAR_ENDPOINT: &'static str = "https://space.galaxybuster.net/lib/star.php";

const UNSTAR_ENDPOINT: &'static str = "https://space.galaxybuster.net/lib/unstar.php";

pub struct SpaceEmailClient {
    http_client: hyper::Client,
    session_id: RefCell<Option<String>>,
}

impl SpaceEmailClient {
    fn make_request(&self, endpoint: &str, parameters: &[(&str, &str)]) -> Result<String, SpaceEmailError> {

        use hyper::header::Headers;
        use hyper::header::{ContentType, UserAgent, Cookie, SetCookie};
        use regex::Regex;

        lazy_static! {
            static ref SESSION_REGEX: Regex = Regex::new("PHPSESSID=([0-9a-f]+)").unwrap();
        }

        fn form_encode(pairs: &[(&str, &str)]) -> String {
            use url;
            url::form_urlencoded::Serializer::new(String::new()).extend_pairs(pairs).finish()
        }
        
        let mut headers = Headers::new();
        headers.set(ContentType("application/x-www-form-urlencoded".parse().unwrap()));
        headers.set(UserAgent("SpaceEmail API Client".to_string()));

        if let Some(id) = self.session_id.borrow().clone() {
            headers.set(Cookie(vec![format!("PHPSESSID={}", id)]));
        }

        match self.http_client.post(endpoint)
            .headers(headers)
            .body(&form_encode(parameters))
            .send()
        {
            Ok(mut response) => {
                if let Some(&SetCookie(ref cookies)) = response.headers.get() {
                    if let Some(id) = SESSION_REGEX.captures(&cookies[0]).and_then(|c| c.get(1)) {
                        *self.session_id.borrow_mut() = Some(id.as_str().to_string());
                    }
                }

                let mut response_body = String::new();
                match response.read_to_string(&mut response_body) { 
                    Ok(_) => Ok(response_body),
                    Err(_) => Err(SpaceEmailError::MalformedResponse("Response encoding error".to_string()))
                }
            },
            Err(e) => Err(SpaceEmailError::Network(e))
        }
    }
}

impl SpaceEmailClient {
    
    pub fn new() -> SpaceEmailClient {
        use hyper::net::HttpsConnector;
        use hyper_native_tls::NativeTlsClient;

        SpaceEmailClient {
            http_client: Client::with_connector(HttpsConnector::new(NativeTlsClient::new().unwrap())),
            session_id: RefCell::new(None)
        }
    }
    
    pub fn get_random(&self) -> Result<SpaceEmail, SpaceEmailError> {
        self.get_random_with_range(3)
    }
    
    pub fn get_random_with_range(&self, range: u32) -> Result<SpaceEmail, SpaceEmailError> {
        fn parse_get(response: String) -> Result<(u32, SpaceEmailColor), SpaceEmailError> {
            let get_fragment = Document::from(&*response);
            match get_fragment.find(Class("row-message")).nth(0).map(
                |n| {(
                        n.attr("data-id").map(str::parse),
                        n.attr("class").and_then(|classes| classes.split_whitespace().find(|class| class.starts_with("msg-"))).map_or(SpaceEmailColor::None, SpaceEmailColor::from_css)
                    )}
            ) {
                Some((Some(Ok(i)), c)) => Ok((i, c)),
                _ => Err(SpaceEmailError::MalformedResponse("Unable to parse view response".to_string()))
            }
        }

        let (id, color) = match self.make_request(GET_ENDPOINT, &[("range", &range.to_string())]) {
            Ok(r) => match parse_get(r) {
                Ok(g) => g,
                Err(e) => return Err(e)
            },
            Err(e) => return Err(e)
        };
        
        let email = match self.get_id(id) {
            Ok(m) => m,
            Err(e) => return Err(e)
        };

        let mut contents = email.contents().clone();
        contents.color = color;

        Ok(create_space_email(
            email.id(),
            email.share_id(),
            email.timestamp(),
            contents
        ))
    }

    pub fn get_id(&self, id: u32) -> Result<SpaceEmail, SpaceEmailError> {
        fn parse_view(response: String, id: u32) -> Result<SpaceEmail, SpaceEmailError> {
            use serde_json;
            fn parse_timestamp(date: &str) -> Result<chrono::NaiveDateTime, chrono::format::ParseError> {
                (chrono::NaiveDateTime::parse_from_str(date, "%A, %b %eth, %Y at %I:%M%P")).or
                (chrono::NaiveDateTime::parse_from_str(date, "%A, %b %est, %Y at %I:%M%P")).or
                (chrono::NaiveDateTime::parse_from_str(date, "%A, %b %end, %Y at %I:%M%P")).or
                (chrono::NaiveDateTime::parse_from_str(date, "%A, %b %erd, %Y at %I:%M%P"))
            }

            let response_data: [String; 3] = match serde_json::from_str(&response) {
                Ok(r) => r,
                Err(_) => return Err(SpaceEmailError::MalformedResponse(format!("Unable to parse view response to JSON: {}", response)))
            };

            let view_fragment = Document::from(&*response_data[0]);
            println!("View response: {}", &response_data[0]);

            let subject = match view_fragment.find(Attr("id", "msgSubject")).nth(0).map(|n| n.text().trim().to_string()) {
                Some(t) => t,
                None => return Err(SpaceEmailError::MalformedResponse("Subject not found".to_string()))
            };

            let sender = match view_fragment.find(Attr("id", "msgSender")).nth(0).map(|n| n.text().trim().to_string()) {
                Some(t) => t,
                None => return Err(SpaceEmailError::MalformedResponse("Sender not found".to_string()))
            };

            let body = match view_fragment.find(Attr("id", "msgBody")).nth(0).map(|n| n.text().trim().to_string()) {
                Some(t) => t,
                None => return Err(SpaceEmailError::MalformedResponse("Body not found".to_string()))
            };

            let timestamp = match view_fragment.find(Attr("id", "msgDate")).nth(0).map(|n| parse_timestamp(n.text().as_str().trim())) {
                Some(Ok(t)) => t,
                _ => return Err(SpaceEmailError::MalformedResponse("Timestamp not found".to_string()))
            };

            Ok(create_space_email(
                id,
                &response_data[2],
                timestamp,
                SpaceEmailContents {
                    subject: subject.to_string(),
                    sender: sender.to_string(),
                    body: body.to_string(),
                    color: SpaceEmailColor::None,
                }
            ))
        }

        match self.make_request(VIEW_ENDPOINT, &[("id", &id.to_string())]) {
            Ok(r) => parse_view(r, id),
            Err(e) => Err(e)
        }
    }
    
    pub fn send(&self, email: &SpaceEmailContents) -> Result<(), SpaceEmailError> {
        if email.sender == "" || email.subject == "" || email.body == "" {
            return Err(SpaceEmailError::InvalidParameter)
        }
        
        match self.make_request(SEND_ENDPOINT, &[
            ("sender", &email.sender), 
            ("subject", &email.subject), 
            ("body", &email.body), 
            ("type", &email.color.into_id().to_string())])
        {
            Ok(r) => {
                match &*r {
                    "wrap success" => Ok(()),
                    _ => Err(SpaceEmailError::InvalidParameter)
                }
            }
            Err(e) => Err(e)
        }
    }
    
}