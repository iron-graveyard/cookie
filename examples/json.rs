#![feature(phase)]
#[phase(plugin, link)] extern crate log;
extern crate http;
extern crate iron;
extern crate cookie;
extern crate serialize;

use std::collections::HashMap;
use std::io::net::ip::Ipv4Addr;
use serialize::json::Number;
use http::status::Ok;
use iron::{Iron, ServerT, Request, Response, Alloy};
use iron::middleware::{Status, Continue};
use iron::mixin::Serve;
use cookie::{CookieParser, Cookie, SetCookie};

fn count_views(_req: &mut Request, res: &mut Response, alloy: &mut Alloy) -> Status {
    // Only hold on to cookies for ten seconds
    let mut options = HashMap::new();
    options.insert("Max-Age".to_string(), Some("10".to_string()));

    match alloy.find::<Cookie>() {
        Some(cookie) => {
            // Find the "count" cookie
            debug!("COOKIE struct: {}", cookie)
            match cookie.json.find(&"count".to_string()) {
                // Grab the string from the json's `count` key
                Some(&Number(mut cnt)) => {
                    // Increment our cookie counter
                    cnt = cnt + 1f64;
                    let count = (cnt).to_str().clone();
                    println!("COOKIE COUNT: {}", count)
                    // Override the cookie with a new value
                    res.set_json_cookie(cookie, ("count".to_string(), Number(cnt)), &options);
                    let _ = res.serve(Ok, format!("Hit Counter: {}", count).as_slice());
                },
                _       => {
                    // Initialize our cookie counter
                    res.set_json_cookie(cookie, ("count".to_string(), Number(1f64)), &options);
                    let _ = res.serve(Ok, format!("Hit Counter: {}", 1f64).as_slice());
                }
            }
        },
        _            => {} // This should never occur, so long as the CookieParser is linked first
    }
    Continue
}

fn main() {
    let mut server: ServerT = Iron::new();
    server.link(CookieParser::signed("@zzmp".to_string()));
    server.link(count_views);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
