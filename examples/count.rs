extern crate http;
extern crate iron;
extern crate cookie;

use std::collections::HashMap;
use std::io::net::ip::Ipv4Addr;
use http::status::Ok;
use iron::{Iron, ServerT, Request, Response, Alloy};
use iron::middleware::{Status, Continue};
use iron::mixin::Serve;
use cookie::{CookieParser, Cookie, SetCookie};

fn count_views(_req: &mut Request, res: &mut Response, alloy: &mut Alloy) -> Status {
    let mut options = HashMap::new();
    options.insert("Max-Age".to_string(), Some("10".to_string()));

    match alloy.find::<Cookie>() {
        // Increment our cookie counter
        Some(cookie) => {
            match cookie.map.find(&"count".to_string()) {
                Some(&Some(ref i)) => {
                    let cnt: Option<int> = from_str(i.as_slice());
                    let count = (cnt.unwrap() + 1).to_str().clone();
                    res.set_cookie(("count".to_string(), count.clone()), &options);
                    let _ = res.serve(Ok, format!("Hit Counter: {}", count).as_slice());
                    return Continue;
                },
                _             => ()
            }
        },
        _            => ()
    }
    // Initialize new cookies
    res.set_cookie(("count".to_string(), "1".to_string()), &options);
    let _ = res.serve(Ok, "Hit Counter: 1");
    Continue
}

fn main() {
    let mut server: ServerT = Iron::new();
    server.smelt(CookieParser::new());
    server.smelt(count_views);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
