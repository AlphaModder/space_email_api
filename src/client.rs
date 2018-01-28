use super::data::*;
use hyper;
use chrono;
use select::document::Document;
use select::predicate::{Class, Attr};
use std::io::Read;
use hyper::Client;
use std::cell::RefCell;

const LOGIN_ENDPOINT: &str = "https://space.galaxybuster.net/login.php";

const LOGOUT_ENDPOINT: &str = "https://space.galaxybuster.net/logout.php";

const GET_ENDPOINT: &str = "https://space.galaxybuster.net/lib/get.php";

const VIEW_ENDPOINT: &str = "https://space.galaxybuster.net/lib/view.php"; 

const SEND_ENDPOINT: &str = "https://space.galaxybuster.net/lib/send.php";

const STAR_ENDPOINT: &str = "https://space.galaxybuster.net/lib/star.php";

const UNSTAR_ENDPOINT: &str = "https://space.galaxybuster.net/lib/unstar.php";

const PAGINATE_ENDPOINT: &str = "https://space.galaxybuster.net/lib/paginatestar.php";

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
    
    pub fn login(&self, email: &str, password: &str) -> Result<(), SpaceEmailError> {
        match self.make_request(LOGIN_ENDPOINT, &[
            ("email", email),
            ("password", password),
        ]) {
            Ok(r) => match &*r { 
                "" => Ok(()),
                _ => Err(SpaceEmailError::InvalidParameter),
            },
            Err(e) => Err(e),
        }
    }

    pub fn logout(&self) -> Result<(), SpaceEmailError> {
        self.make_request(LOGOUT_ENDPOINT, &[])?;
        Ok(())
    }

    pub fn get_random(&self) -> Result<SpaceEmail, SpaceEmailError> {
        self.get_random_with_range(SpaceEmailRange::All)
    }
    
    pub fn get_random_in_range(&self, range: SpaceEmailRange) -> Result<SpaceEmail, SpaceEmailError> {
        fn parse_get(response: String) -> Result<(u32, SpaceEmailColor), SpaceEmailError> {
            let get_fragment = Document::from(&*response);
            match get_fragment.find(Class("row-message")).nth(0).map(
                |n| {(
                        n.attr("data-id").map(str::parse),
                        n.attr("class").and_then(|classes| classes.split_whitespace().find(|class| class.starts_with("msg-"))).map_or(SpaceEmailColor::Yellow, SpaceEmailColor::from_css)
                    )}
            ) {
                Some((Some(Ok(i)), c)) => Ok((i, c)),
                _ => Err(SpaceEmailError::MalformedResponse("Unable to parse get response".to_string()))
            }
        }

        let (id, color) = match self.make_request(GET_ENDPOINT, &[("range", &range.into_id().to_string())]) {
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

        Ok(SpaceEmail {
            contents: SpaceEmailContents {
                color: color,
                .. email.contents().clone()
            },
            .. email
        })
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

            Ok(SpaceEmail {
                id: id,
                share_id: response_data[2].clone(),
                timestamp: timestamp,
                contents: SpaceEmailContents {
                    subject: subject.to_string(),
                    sender: sender.to_string(),
                    body: body.to_string(),
                    color: SpaceEmailColor::Yellow,
                }
            })
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
    
    pub fn star(&self, email: &SpaceEmail) -> Result<(), SpaceEmailError> {
        match self.make_request(STAR_ENDPOINT, &[("id", &email.id().to_string())]) {
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        }
    }

    pub fn unstar(&self, email: &SpaceEmail) -> Result<(), SpaceEmailError> {
        match self.make_request(UNSTAR_ENDPOINT, &[("id", &email.id().to_string())]) {
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        }
    }

    pub fn get_starred_emails(&self) -> StarredIterator {
        StarredIterator {
            client: self,
            page: 1,
            buffer: Vec::new(),
        }
    }
    
}

pub struct StarredIterator<'a> {
    client: &'a SpaceEmailClient, 
    page: u32,
    buffer: Vec<(u32, SpaceEmailColor)>,
}

impl<'a> Iterator for StarredIterator<'a> {
    type Item = Result<SpaceEmail, SpaceEmailError>;
    fn next(&mut self) -> Option<Result<SpaceEmail, SpaceEmailError>> {
        fn parse_paginate(response: String, buffer: &mut Vec<(u32, SpaceEmailColor)>) -> Result<(), SpaceEmailError> {
            let paginate_fragment = Document::from(&*response);
            let emails = paginate_fragment
                .find(Class("row-message"))
                .map(
                    |n| {(
                        n.attr("data-id").map(str::parse),
                        n.attr("class").and_then(|classes| classes.split_whitespace().find(|class| class.starts_with("msg-"))).map_or(SpaceEmailColor::Yellow, SpaceEmailColor::from_css)
                    )}
                );
            for email in emails {
                if let (Some(Ok(id)), color) = email {
                    buffer.push((id, color));
                }
                else {
                    return Err(SpaceEmailError::MalformedResponse("Unable to parse paginate response!".to_string()));
                }
            }
            Ok(())
        }

        if self.buffer.is_empty() {
            match self.client.make_request(PAGINATE_ENDPOINT, &[("page", &self.page.to_string())]) {
                Err(e) => return Some(Err(e)),
                Ok(r) => { 
                    if let Err(e) = parse_paginate(r, &mut self.buffer) {
                        return Some(Err(e))
                    } 
                    else {
                        self.page += 1;
                    } 
                }
            }
            if self.buffer.is_empty() {
                return None
            }
        }

        let email_data = self.buffer.remove(0);
        match self.client.get_id(email_data.0) {
            Ok(email) => Some(Ok(SpaceEmail {
                contents: SpaceEmailContents {
                    color: email_data.1,
                    .. email.contents().clone()
                },
                .. email
            })),
            Err(e) => {
                self.buffer.insert(0, email_data);
                Some(Err(e))
            }
        }
    }
}

