use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::pool::Pool;
use crate::server::Server;

enum Command {
    Help,
    Print,
    Unknown,
    Grow(usize),
    Shrink(usize),
}

impl Command {
    fn from(method: &str, count: usize) -> Command {
        match method.to_lowercase().as_str() {
            "grow" => Command::Grow(count),
            "shrink" => Command::Shrink(count),
            "print" => Command::Print,
            "help" => Command::Help,
            _ => Command::Unknown,
        }
    }
}

pub fn start_server(pool: Pool, server: Server, port: u16) {
    let pool = Arc::new(Mutex::new(pool));
    let server_pool = pool.clone();
    let routes = server.router.print();

    thread::spawn(move || server.listen(port, server_pool));

    let pool = || pool.lock().unwrap();

    loop {
        match accept_command() {
            Command::Help => {
                println!("Usage:");
                println!("   grow <n>   - Increase capacity by <n> threads.");
                println!("   shrink <n> - Decrease capacity by <n> threads.");
                println!("   print      - Show the router configuration.");
                println!("   help       - Show this help.");
            }
            Command::Print => println!("{}", routes),
            Command::Grow(by) => println!("{}", pool().grow(by)),
            Command::Shrink(by) => println!("{}", pool().shrink(by)),
            Command::Unknown => println!("Unknown command."),
        }
    }
}

fn accept_command() -> Command {
    print!("web-server# ");
    io::stdout().flush().unwrap();

    let mut cmd = String::new();
    match io::stdin().read_line(&mut cmd) {
        Ok(_) => parse_command(cmd),
        Err(_) => Command::Unknown,
    }
}

fn parse_command(cmd: String) -> Command {
    let mut parts = cmd.trim().split_whitespace();
    if let Some(method) = parts.next() {
        let count = if let Some(count) = parts.next() {
            count.parse::<usize>().unwrap_or(0)
        } else {
            0
        };

        Command::from(method, count)
    } else {
        Command::Unknown
    }
}
