extern crate http;
extern crate iron;
extern crate cookie;

use std::io::net::ip::Ipv4Addr;
use http::status;
use iron::{Iron, ChainBuilder, Chain, Request, Response, IronResult};
use cookie::{CookieParser, Cookie, SetCookie, HeaderCollection};

fn count_views(req: &mut Request) -> IronResult<Response> {
    // Only hold on to cookies for ten seconds
    let options = HeaderCollection::aged(10);

    match req.extensions.find::<CookieParser, Cookie>() {
        Some(cookie) => {
            // Find the "count" cookie
            match cookie.map.find(&"count".to_string()) {
                Some(i) => {
                    // Convert the string to an int
                    let cnt: Option<int> = from_str(i.as_slice());
                    // Increment our cookie counter
                    let count = (cnt.unwrap() + 1).to_string();
                    println!("COOKIE COUNT: {}", i)
                    // Override the cookie with a new value
                    let mut res = Response::with(status::Ok, format!("Hit Counter: {}", count).as_slice());
                    res.set_cookie(cookie, ("count".to_string(), count.clone()), options);
                    return Ok(res);
                },
                _ => {
                    // Initialize our cookie counter
                    let mut res = Response::with(status::Ok, format!("Hit Counter: {}", 1u8).as_slice());
                    res.set_cookie(cookie, ("count".to_string(), "1".to_string()), options);
                    return Ok(res);
                }
            }
        },
        _ => {} // This should never occur, so long as the CookieParser is linked first
    }
    Ok(Response::new())
}

fn main() {
    let mut chain = ChainBuilder::new(count_views);
    chain.link_before(CookieParser::signed("@zzmp".to_string()));

    Iron::new(chain).listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
