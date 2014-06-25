extern crate http;
extern crate iron;
extern crate cookie;

use std::io::net::ip::Ipv4Addr;
use http::status::Ok;
use iron::{Iron, ServerT, Chain, Request, Response, Alloy};
use iron::middleware::{Status, Continue};
use iron::mixin::Serve;
use cookie::{CookieParser, Cookie, SetCookie, HeaderCollection};

fn count_views(_req: &mut Request, res: &mut Response, alloy: &mut Alloy) -> Status {
    // Only hold on to cookies for ten seconds
    let options = HeaderCollection::aged(10);

    match alloy.find::<Cookie>() {
        Some(cookie) => {
            // Find the "count" cookie
            match cookie.map.find(&"count".to_string()) {
                Some(i) => {
                    // Convert the string to an int
                    let cnt: Option<int> = from_str(i.as_slice());
                    // Increment our cookie counter
                    let count = (cnt.unwrap() + 1).to_str().clone();
                    println!("COOKIE COUNT: {}", i)
                    // Override the cookie with a new value
                    res.set_cookie(cookie, ("count".to_string(), count.to_string()), options);
                    let _ = res.serve(Ok, format!("Hit Counter: {}", count).as_slice());
                },
                _       => {
                    // Initialize our cookie counter
                    res.set_cookie(cookie, ("count".to_string(), "1".to_string()), options);
                    let _ = res.serve(Ok, format!("Hit Counter: {}", 1).as_slice());
                }
            }
        },
        _            => {} // This should never occur, so long as the CookieParser is linked first
    }
    Continue
}

fn main() {
    let mut server: ServerT = Iron::new();
    server.chain.link(CookieParser::signed("@zzmp".to_string()));
    server.chain.link(count_views);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
