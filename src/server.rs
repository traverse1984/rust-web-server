use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use crate::request::Request;
use crate::response::*;
use crate::route::Route;

const READ_INIT_TIMEOUT: u64 = 10;
const READ_TIMEOUT_MS: u64 = 10;
const READ_MAX_SIZE: usize = 1024 * 32;

pub struct Server<'a> {
    pub router: Route<'a>,
}

impl<'a> Server<'a> {
    pub fn new() -> &'static mut Server<'static> {
        Box::leak(Box::new(Server {
            router: Route::new(),
        }))
    }

    pub fn listen(&'static self, port: u16) {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(addr).expect("Unable to open listener.");

        for stream in listener.incoming() {
            thread::spawn(move || {
                let mut stream = stream.unwrap();
                let res = self.handle(&mut stream);
                stream.write(res.produce().as_bytes()).unwrap();
            });
        }
    }

    fn bad_request(&self, message: &str) -> HttpResponse {
        let mut res = HttpResponse::new(HttpStatus::BadRequest);
        res.append(message);
        res
    }

    fn handle(&self, stream: &mut TcpStream) -> HttpResponse {
        let mut buf = [0; 4096];
        let mut accept: usize = 4096;
        let mut req = Vec::new();
        let mut rcv_first_bytes = false;

        stream
            .set_read_timeout(Some(Duration::from_secs(READ_INIT_TIMEOUT)))
            .unwrap();

        while let Ok(len) = stream.read(&mut buf) {
            if len == 0 {
                break;
            } else if accept < len {
                return self.bad_request("Request input too large.");
            }

            req.extend_from_slice(&buf[..len]);
            accept = usize::min(READ_MAX_SIZE - req.len(), 4096);

            if !rcv_first_bytes {
                stream
                    .set_read_timeout(Some(Duration::from_millis(READ_TIMEOUT_MS)))
                    .unwrap();
                rcv_first_bytes = true;
            }
        }

        let raw = String::from_utf8_lossy(&req);

        if let Ok(req) = Request::new(&raw) {
            return match self.router.route(req.uri()) {
                Some(handle) => handle(&req),
                None => {
                    let mut res = HttpResponse::new(HttpStatus::NotFound);
                    res.append(format!("Did not find path {}", req.uri()));
                    res
                }
            };
        }

        self.bad_request("Malformed request.")
    }
}
