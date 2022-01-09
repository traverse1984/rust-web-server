mod control;
mod pool;
mod request;
mod response;
mod route;
mod server;

use pool::Pool;
use request::Request;
use response::{HttpResponse, HttpStatus};
use server::Server;

use std::thread;
use std::time::Duration;

fn named_route(name: &'static str) -> impl Fn(&Request) -> HttpResponse {
    move |req: &Request| {
        let mut res = HttpResponse::new(HttpStatus::Ok);
        res.header("Content-Type", "text/html")
            .ln(format!("<h1>Route: {}</h1>", name))
            .ln(format!("<h2>Request URI: {}", req.uri()));
        res
    }
}

fn main() {
    let mut server = Server::new();

    server.router.add("*", named_route("default"));
    server.router.add("/match/*", named_route("/match/*"));
    server
        .router
        .add("/match/specific", named_route("specific"));

    let slow = named_route("slow");
    server.router.add("/slow", move |req| {
        thread::sleep(Duration::from_secs(5));
        slow(req)
    });

    server.router.add("/daisy", move |_| {
        let mut res = HttpResponse::new(HttpStatus::Ok);
        res.append("<h1>Hello Daisy :)</h1>");
        res
    });

    control::start_server(Pool::new(4), server, 4000);
}
