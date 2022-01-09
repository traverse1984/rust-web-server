use std::collections::HashMap;
use std::fmt::Display;

pub struct HttpResponse {
    status: HttpStatus,
    body: String,
    headers: HashMap<&'static str, String>,
}

impl HttpResponse {
    pub fn new(status: HttpStatus) -> HttpResponse {
        HttpResponse {
            status,
            body: String::new(),
            headers: HashMap::new(),
        }
    }

    pub fn header(&mut self, key: &'static str, value: impl Display) -> &mut Self {
        if key.trim().to_lowercase() != "content-length" {
            self.headers.insert(key.trim(), format!("{}", value));
        }
        self
    }

    pub fn append<T: Into<String>>(&mut self, body: T) -> &mut Self {
        self.body.push_str(&body.into());
        self
    }

    pub fn ln<T: Into<String>>(&mut self, body: T) -> &mut Self {
        self.body.push_str(&body.into());
        self.body.push('\n');
        self
    }

    pub fn produce(self) -> String {
        let mut res = String::from(self.status.as_str());
        res.push('\n');
        res.push_str(format!("Content-Length: {}\n", self.body.len()).as_str());

        for (&key, value) in &self.headers {
            res.push_str(format!("{}: {}\n", key, value).as_str());
        }

        res.push('\n');
        res.push_str(self.body.as_str());
        res
    }
}

pub enum HttpStatus {
    Ok,
    BadRequest,
    Forbidden,
    NotFound,
    InternalServerError,
}

impl HttpStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Ok => "HTTP/1.1 200 OK",
            Self::BadRequest => "HTTP/1.1 400 Bad Request",
            Self::Forbidden => "HTTP/1.1 403 Forbidden",
            Self::NotFound => "HTTP/1.1 404 Not Found",
            Self::InternalServerError => "HTTP/1.1 500 Internal Server Error",
        }
    }
}
