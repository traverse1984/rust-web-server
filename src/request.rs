use std::collections::HashMap;
use std::str::Lines;

pub struct Request<'a> {
    raw: &'a str,
    uri: &'a str,
    headers: HashMap<String, &'a str>,
}

impl<'a> Request<'a> {
    pub fn uri(&self) -> &str {
        return self.uri;
    }

    pub fn header(&self, key: &str) -> &'a str {
        if let Some(value) = self.get(key) {
            value
        } else {
            ""
        }
    }

    pub fn get(&self, key: &str) -> Option<&'a str> {
        let value = self.headers.get(&key.to_lowercase())?;
        Some(*value)
    }

    pub fn new(raw: &'a str) -> Result<Request<'a>, ()> {
        let mut lines = raw.lines();

        if let Some(head) = lines.next() {
            Ok(Request {
                raw,
                uri: Self::parse_get_request(head)?,
                headers: Self::parse_headers(&mut lines)?,
            })
        } else {
            Err(())
        }
    }

    fn parse_get_request(req: &str) -> Result<&str, ()> {
        let req = req.split(' ').enumerate();
        let mut uri: &str = "";

        for (index, segment) in req {
            match (index, segment) {
                (0, "GET") | (2, "HTTP/1.1") => continue,
                (1, _) if segment != "" => uri = segment,
                (_, _) => return Err(()),
            }
        }

        return if uri == "" { Err(()) } else { Ok(uri) };
    }

    fn parse_headers(req: &mut Lines<'a>) -> Result<HashMap<String, &'a str>, ()> {
        let mut headers = HashMap::new();
        for header in req {
            match header.split_once(':') {
                None if header == "" => return Ok(headers),
                Some((key, value)) => headers.insert(key.trim().to_lowercase(), value.trim()),
                _ => return Err(()),
            };
        }

        Ok(headers)
    }
}
