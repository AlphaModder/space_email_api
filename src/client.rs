use reqwest::{Client, ClientBuilder, header::{HeaderMap, USER_AGENT}};
use select::document::Document;
use select::predicate::{Class, Attr};
use futures::prelude::*;
use async_stream::stream;

use crate::data::*;

const LOGIN_ENDPOINT: &'static str = "https://space.galaxybuster.net/login.php";

const LOGOUT_ENDPOINT: &'static str = "https://space.galaxybuster.net/logout.php";

const GET_ENDPOINT: &'static str = "https://space.galaxybuster.net/lib/get.php";

const VIEW_ENDPOINT: &'static str = "https://space.galaxybuster.net/lib/view.php"; 

const SEND_ENDPOINT: &'static str = "https://space.galaxybuster.net/lib/send.php";

const STAR_ENDPOINT: &'static str = "https://space.galaxybuster.net/lib/star.php";

const UNSTAR_ENDPOINT: &'static str = "https://space.galaxybuster.net/lib/unstar.php";

const PAGINATE_ENDPOINT: &'static str = "https://space.galaxybuster.net/lib/paginatestar.php";

pub type Result<T> = std::result::Result<T, SpaceEmailError>;

/// A client capable of sending and downloading space emails, as well as managing the stars
/// of a space email account.
pub struct SpaceEmailClient {
    client: Client,
    logged_in: bool,
}

impl SpaceEmailClient {
    /// Creates a new `SpaceEmailClient`. This method returns `Err` if and only if
    /// it is unable to create a new `reqwest::Client` internally.
    pub fn new() -> Result<SpaceEmailClient> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, "Space Email API Client".parse().unwrap());
        Ok(SpaceEmailClient {
            client: ClientBuilder::new().default_headers(headers).cookie_store(true).build()?,
            logged_in: false,
        })
    }

    /// Attempts to login to space email with the provided credentials.
    pub async fn login(&mut self, email: &str, password: &str) -> Result<()> {
        let params = [("email", email), ("password", password)];
        match self.client.post(LOGIN_ENDPOINT).form(&params).send().await?.text().await?.as_ref() {
            "" => { self.logged_in = true; Ok(()) },
            _ => Err(SpaceEmailError::InvalidParameter)
        }
    }

    /// Attempts to log out of space email.
    pub async fn logout(&mut self) -> Result<()> {
        self.client.post(LOGOUT_ENDPOINT).send().await?;
        self.logged_in = false;
        Ok(())
    }

    /// Attempts to download a random space email.
    pub async fn get_random(&self) -> Result<SpaceEmail> {
        self.get_random_in_range(SpaceEmailRange::All).await
    }

    /// Attempts to download a random space email sent during the time range specified by `range`. If this client
    /// is not logged in, range must equal `SpaceEmailRange::All`, else `Err(SpaceEmailError::RequiresLogin)` will be returned.
    pub async fn get_random_in_range(&self, range: SpaceEmailRange) -> Result<SpaceEmail> {
        if !self.logged_in && range != SpaceEmailRange::All {
            return Err(SpaceEmailError::RequiresLogin)
        }

        let params = [("range", range.into_id())];
        let response = self.client.post(GET_ENDPOINT).form(&params).send().await?.text().await?;

        let get_fragment = Document::from(&*response);

        let id = match get_fragment.find(Class("row-message")).nth(0).map(|n| (
            n.attr("data-id").map(str::parse),
            n.attr("class").and_then(
                |classes| classes.split_whitespace().find(|class| class.starts_with("msg-") || *class == "admin")
            ).map_or(SpaceEmailStyle::Yellow, SpaceEmailStyle::from_css)
        )) {
            Some((Some(Ok(id)), style)) => EmailId { id, style },
            _ => return Err(SpaceEmailError::MalformedResponse("Unable to parse get response".to_string()))
        };

        self.get_by_id(id).await
    }

    /// Attempts to download the space email with id `id`. Due to technical limitations, it is not always 
    /// possible to detect the email's proper color with this method, and emails with a `style` of 
    /// `SpaceEmailStyle::Yellow` may be returned when the email's actual color differs.
    pub async fn get_by_id(&self, id: impl Into<EmailId>) -> Result<SpaceEmail> {
        let id = id.into();
        let response = self.client.post(VIEW_ENDPOINT).form(&[("id", id.id)]).send().await?.text().await?;
        let response_data: [String; 3] = match serde_json::from_str(&response) {
            Ok(r) => r,
            Err(_) => return Err(SpaceEmailError::MalformedResponse(format!("Unable to parse view response to JSON: {}", response)))
        };

        let view_fragment = Document::from(&*response_data[0]);

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
            id: id.id,
            share_id: response_data[2].clone(),
            timestamp: timestamp,
            contents: SpaceEmailContents {
                subject: subject.to_string(),
                sender: sender.to_string(),
                body: body.to_string(),
                style: id.style,
            }
        })
    }

    /// Attempts to send an email with the specified `SpaceEmailContents`.
    pub async fn send(&self, email: &SpaceEmailContents) -> Result<()> {
        if email.sender == "" || email.subject == "" || email.body == "" || email.style == SpaceEmailStyle::Admin {
            return Err(SpaceEmailError::InvalidParameter)
        }

        if !self.logged_in && email.style != SpaceEmailStyle::Yellow {
            return Err(SpaceEmailError::RequiresLogin)
        }

        let params = [
            ("sender", &email.sender),
            ("subject", &email.subject),
            ("body", &email.body),
            ("type", &email.style.into_id().unwrap().to_string())
        ];

        match self.client.post(SEND_ENDPOINT).form(&params).send().await?.text().await?.as_ref() {
            "wrap success" => Ok(()),
            _ => Err(SpaceEmailError::InvalidParameter),
        }
    }

    /// Attempts to star the specified email. Returns `Err(SpaceEmailError::RequiresLogin)` if the
    /// client is not logged in.
    pub async fn star(&self, email: impl Into<EmailId>) -> Result<()> {
        if !self.logged_in { return Err(SpaceEmailError::RequiresLogin) }
        self.client.post(STAR_ENDPOINT).form(&[("id", email.into().id)]).send().await?;
        Ok(())
    }

    /// Attempts to unstar the specified email. Returns `Err(SpaceEmailError::RequiresLogin)` if the
    /// client is not logged in.
    pub async fn unstar(&self, email: impl Into<EmailId>) -> Result<()> {
        if !self.logged_in { return Err(SpaceEmailError::RequiresLogin) }
        self.client.post(UNSTAR_ENDPOINT).form(&[("id", email.into().id)]).send().await?;
        Ok(())
    }

    /// Provides a stream of `EmailId`s corresponding to emails starred by the currently logged-in user. To
    /// obtain the emails themselves, pass the result of this method to `SpaceEmailClient::get_by_id`.  
    /// Returns `Err(SpaceEmailError::RequiresLogin)` if the client is not logged in.
    pub fn starred_emails<'a>(&'a self) -> Result<impl Stream<Item=Result<EmailId>> + 'a> {
        if !self.logged_in { return Err(SpaceEmailError::RequiresLogin) }
        let stream = stream! {
            let mut page: usize = 1;
            loop {
                let emails = match self.client.post(PAGINATE_ENDPOINT).form(&[("page", page)]).send().await {
                    Ok(response) => match response.text().await {
                        Ok(text) => Document::from(&*text)
                            .find(Class("row-message"))
                            .map(|n| (
                                n.attr("data-id").map(str::parse),
                                n.attr("class")
                                    .and_then(|classes| classes.split_whitespace().find(|class| class.starts_with("msg-")))
                                    .map_or(SpaceEmailStyle::Yellow, SpaceEmailStyle::from_css)
                            )).collect::<Vec<_>>(),
                        Err(e) => { yield Err(e.into()); continue }
                    }
                    Err(e) => { yield Err(e.into()); continue }
                };

                let emails = emails.into_iter().fold(Ok(Vec::new()), |buf, (id, style)| {
                    buf.and_then(|mut buf| match id {
                        Some(Ok(id)) => { buf.push(EmailId { id, style }); Ok(buf) }
                        _ => Err(SpaceEmailError::MalformedResponse("Unable to parse paginate response.".to_string()))
                    })
                });

                match emails {
                    Ok(emails) if emails.is_empty() => return,
                    Ok(emails) => for email in emails { yield Ok(email); }
                    Err(e) => { yield Err(e); continue }
                }

                page += 1;
            }
        };
        Ok(stream)
    }
}

fn parse_timestamp(date: &str) -> std::result::Result<chrono::NaiveDateTime, chrono::format::ParseError> {
    (chrono::NaiveDateTime::parse_from_str(date, "%A, %b %eth, %Y at %I:%M%P")).or
    (chrono::NaiveDateTime::parse_from_str(date, "%A, %b %est, %Y at %I:%M%P")).or
    (chrono::NaiveDateTime::parse_from_str(date, "%A, %b %end, %Y at %I:%M%P")).or
    (chrono::NaiveDateTime::parse_from_str(date, "%A, %b %erd, %Y at %I:%M%P"))
}