use super::data::*;

use hyper;
use chrono;

use futures;
use futures::prelude::*;

use select::document::Document;
use select::predicate::{Class, Attr};

use std::cell::{Cell, RefCell};
use std::str;

use hyper::Uri;

type SpaceFuture<'a, I> = Box<Future<Item=I, Error=SpaceEmailError> + 'a>;

lazy_static! {
    static ref LOGIN_ENDPOINT: Uri = "https://space.galaxybuster.net/login.php".parse().unwrap();

    static ref LOGOUT_ENDPOINT: Uri = "https://space.galaxybuster.net/logout.php".parse().unwrap();

    static ref GET_ENDPOINT: Uri = "https://space.galaxybuster.net/lib/get.php".parse().unwrap();

    static ref VIEW_ENDPOINT: Uri = "https://space.galaxybuster.net/lib/view.php".parse().unwrap(); 

    static ref SEND_ENDPOINT: Uri = "https://space.galaxybuster.net/lib/send.php".parse().unwrap();

    static ref STAR_ENDPOINT: Uri = "https://space.galaxybuster.net/lib/star.php".parse().unwrap();

    static ref UNSTAR_ENDPOINT: Uri = "https://space.galaxybuster.net/lib/unstar.php".parse().unwrap();

    static ref PAGINATE_ENDPOINT: Uri = "https://space.galaxybuster.net/lib/paginatestar.php".parse().unwrap();
}

pub struct SpaceEmailClient<C: hyper::client::Connect> {
    http_client: hyper::Client<C>,
    session_id: RefCell<Option<String>>,
    logged_in: Cell<bool>,
}

impl<C: hyper::client::Connect> SpaceEmailClient<C> {
    fn make_request<'a>(&'a self, endpoint: &Uri, parameters: &[(&str, &str)]) -> SpaceFuture<'a, String> {
        fn form_encode(pairs: &[(&str, &str)]) -> String {
            ::url::form_urlencoded::Serializer::new(String::new()).extend_pairs(pairs).finish()
        }

        use regex::Regex;

        lazy_static! {
            static ref SESSION_REGEX: Regex = Regex::new("PHPSESSID=([0-9a-f]+)").unwrap();
        }

        use hyper::{Request, Method};
        use hyper::header::{ContentType, UserAgent, Cookie, SetCookie};

        let mut request = Request::new(Method::Post, endpoint.clone());
        {
            let headers = request.headers_mut();
            headers.set(ContentType::form_url_encoded());
            headers.set(UserAgent::new("SpaceEmail API Client"));
            if let Some(id) = self.session_id.borrow().clone() {
                let mut cookie = Cookie::new();
                cookie.append("PHPSESSID", id);
                headers.set(cookie);
            }
        }
        request.set_body(form_encode(parameters));

        Box::new(
            self.http_client.request(request).from_err::<SpaceEmailError>().and_then(move |r| {
                if let Some(&SetCookie(ref cookies)) = r.headers().get() {
                    if let Some(id) = SESSION_REGEX.captures(&cookies[0]).and_then(|c| c.get(1)) {
                        *self.session_id.borrow_mut() = Some(id.as_str().to_string());
                    }
                }
                r.body().concat2().from_err::<SpaceEmailError>()
            }).and_then(|data| Ok(str::from_utf8(&data)?.to_owned()))
        )
    }
}

impl<C: hyper::client::Connect> SpaceEmailClient<C> {
    
    pub fn new(http_client: hyper::Client<C>) -> SpaceEmailClient<C> {
        SpaceEmailClient {
            http_client: http_client,
            session_id: RefCell::new(None),
            logged_in: Cell::new(false),
        }
    }
    
    pub fn login(&self, email: &str, password: &str) -> SpaceFuture<()> {
        Box::new(self.make_request(&LOGIN_ENDPOINT, &[
            ("email", email),
            ("password", password),
        ]).and_then(move |r| match &*r { 
            "" => { self.logged_in.set(true); Ok(()) }
            _ => Err(SpaceEmailError::InvalidParameter),
        }))
    }

    pub fn logout(&self) -> SpaceFuture<()> {
        Box::new(self.make_request(&LOGOUT_ENDPOINT, &[]).map(move |r| { self.logged_in.set(false); }))
    }

    pub fn get_random(&self) -> SpaceFuture<SpaceEmail> {
        self.get_random_in_range(SpaceEmailRange::All)
    }
    
    pub fn get_random_in_range(&self, range: SpaceEmailRange) -> SpaceFuture<SpaceEmail> {
        fn parse_get(response: String) -> Result<(u32, SpaceEmailStyle), SpaceEmailError> {
            let get_fragment = Document::from(&*response);
            match get_fragment.find(Class("row-message")).nth(0).map(
                |n| {(
                        n.attr("data-id").map(str::parse),
                        n.attr("class").and_then(|classes| classes.split_whitespace().find(|class| class.starts_with("msg-") || *class == "admin")).map_or(SpaceEmailStyle::Yellow, SpaceEmailStyle::from_css)
                    )}
            ) {
                Some((Some(Ok(i)), c)) => Ok((i, c)),
                _ => Err(SpaceEmailError::MalformedResponse("Unable to parse get response".to_string()))
            }
        }

        Box::new(self.make_request(&GET_ENDPOINT, &[("range", &range.into_id().to_string())])
            .and_then(|r| { parse_get(r) })
            .and_then(move |(id, style)| { 
                self.get_by_id(id).map(move |email| {
                    SpaceEmail {
                        contents: SpaceEmailContents {
                            style: style,
                            .. email.contents().clone()
                        },
                        .. email
                    }
                })
            })
        )
    }

    pub fn get_by_id(&self, id: u32) -> SpaceFuture<SpaceEmail> {
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
                    style: SpaceEmailStyle::Yellow,
                }
            })
        }

        Box::new(self.make_request(&VIEW_ENDPOINT, &[("id", &id.to_string())]).and_then(move |r| parse_view(r, id)))
    }
    
    pub fn send(&self, email: &SpaceEmailContents) -> SpaceFuture<()> {
        let email = email.clone();
        let email2 = email.clone(); // this is a terrible hack but futures are annoying
        Box::new(futures::lazy(move || {
            if email.sender == "" || email.subject == "" || email.body == "" || email.style == SpaceEmailStyle::Admin {
                return Err(SpaceEmailError::InvalidParameter)
            }

            if !self.logged_in.get() && email.style != SpaceEmailStyle::Yellow {
                return Err(SpaceEmailError::RequiresLogin)
            }
            Ok(())
        }).and_then(move |_| {
            self.make_request(&SEND_ENDPOINT, &[
                ("sender", &email2.sender), 
                ("subject", &email2.subject), 
                ("body", &email2.body), 
                ("type", &email2.style.into_id().unwrap().to_string())
            ])
        }).and_then(|r| {
            match &*r {
                "wrap success" => Ok(()),
                _ => Err(SpaceEmailError::InvalidParameter)
            }
        }))
    }
    
    pub fn star(&self, email: &SpaceEmail) -> SpaceFuture<()> {
        let id = email.id;
        Box::new(futures::lazy(
            move || {if !self.logged_in.get() { Err(SpaceEmailError::RequiresLogin) } else { Ok(()) }
        }).and_then(move |_| {
            self.make_request(&STAR_ENDPOINT, &[("id", &id.to_string())]).map(|_| ())
        }))
    }

    pub fn unstar(&self, email: &SpaceEmail) -> SpaceFuture<()> {
        let id = email.id;
        Box::new(futures::lazy(
            move || {if !self.logged_in.get() { Err(SpaceEmailError::RequiresLogin) } else { Ok(()) }
        }).and_then(move |_| {
            self.make_request(&UNSTAR_ENDPOINT, &[("id", &id.to_string())]).map(|_| ())
        }))
    }

    pub fn starred_emails(&self) -> StarredIterator<C> {
        StarredIterator {
            client: self,
            page: 1,
            buffer: Vec::new(),
            state: Mappable::new(StarredIteratorState::Draining),
        }
    }
}


pub struct StarredIterator<'a, C: hyper::client::Connect> {
    client: &'a SpaceEmailClient<C>, 
    page: u32,
    buffer: Vec<(u32, SpaceEmailStyle)>,
    state: Mappable<StarredIteratorState<'a>>,
}

enum StarredIteratorState<'a> {
    Buffering(SpaceFuture<'a, Vec<(u32, SpaceEmailStyle)>>),
    Draining,
    Getting(SpaceFuture<'a, SpaceEmail>),
}

impl<'a, C: hyper::client::Connect> Stream for StarredIterator<'a, C> {
    type Item = SpaceEmail;
    type Error = SpaceEmailError;
    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        if !self.client.logged_in.get() { return Err(SpaceEmailError::RequiresLogin) }
        fn parse_paginate(response: String) -> Result<Vec<(u32, SpaceEmailStyle)>, SpaceEmailError> {
            let paginate_fragment = Document::from(&*response);
            let mut buffer = Vec::new();
            let emails = paginate_fragment
                .find(Class("row-message"))
                .map(
                    |n| {(
                        n.attr("data-id").map(str::parse),
                        n.attr("class").and_then(|classes| classes.split_whitespace().find(|class| class.starts_with("msg-"))).map_or(SpaceEmailStyle::Yellow, SpaceEmailStyle::from_css)
                    )}
                );
            for email in emails {
                if let (Some(Ok(id)), style) = email {
                    buffer.push((id, style));
                }
                else {
                    return Err(SpaceEmailError::MalformedResponse("Unable to parse paginate response.".to_string()));
                }
            }
            Ok(buffer)
        }

        let mut ret = Ok(Async::NotReady);
        let StarredIterator {ref mut client, ref mut page, ref mut buffer, ref mut state} = self;
        state.map(|state| {
            match state {
                StarredIteratorState::Draining => {
                    if buffer.is_empty() {
                        StarredIteratorState::Buffering(
                            Box::new(client.make_request(&PAGINATE_ENDPOINT, &[("page", &page.to_string())]).and_then(parse_paginate))
                        )
                    }
                    else {
                        let email_data = buffer[0];
                        StarredIteratorState::Getting(
                            Box::new(client.get_by_id(email_data.0).then(move |result| {
                                match result {
                                    Ok(email) => Ok(SpaceEmail {
                                        contents: SpaceEmailContents {
                                            style: email_data.1,
                                            .. email.contents().clone()
                                        },
                                        .. email
                                    }),
                                    Err(e) => {
                                        Err(e)
                                    }
                                }
                            }))
                        )
                    }
                }
                StarredIteratorState::Buffering(mut future) => {
                    match future.poll() {
                        Ok(Async::Ready(v)) => {
                            if !v.is_empty() {
                                *buffer = v;
                                *page += 1;
                                StarredIteratorState::Draining
                            }
                            else {
                                ret = Ok(Async::Ready(None));
                                StarredIteratorState::Buffering(future)
                            }
                        }
                        Ok(Async::NotReady) => StarredIteratorState::Buffering(future),
                        Err(e) => {ret = Err(e); StarredIteratorState::Buffering(future)},
                    }
                }
                StarredIteratorState::Getting(mut future) => {
                    match future.poll() {
                        Ok(Async::Ready(email)) => { 
                            buffer.remove(0);
                            ret = Ok(Async::Ready(Some(email))); 
                            StarredIteratorState::Draining 
                        },
                        Ok(Async::NotReady) => StarredIteratorState::Getting(future),
                        Err(e) => {ret = Err(e); StarredIteratorState::Getting(future)},
                    }
                }
            }
        });
        ret
    }
}

struct Mappable<T>(Option<T>);

impl<T> Mappable<T> {
    fn new(value: T) -> Mappable<T> { Mappable(Some(value)) }

    fn map<F: FnOnce(T) -> T>(&mut self, f: F) {
        self.0 = Some(f(self.0.take().unwrap()));
    }
}