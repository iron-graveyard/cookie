#![feature(phase)]
#[phase(plugin, link)] extern crate log;
extern crate http;
extern crate iron;
extern crate cookie;
extern crate serialize;

use std::io::net::ip::Ipv4Addr;
use serialize::json::U64;
use http::status;
use iron::{Iron, ChainBuilder, Chain, Request, Response, IronResult};
use cookie::{CookieParser, Cookie, SetCookie, HeaderCollection};

fn count_views(req: &mut Request) -> IronResult<Response> {
    // Only hold on to cookies for ten seconds
    let options = HeaderCollection::aged(10);

    match req.extensions.find::<CookieParser, Cookie>() {
        Some(cookie) => {
            // Find the "count" cookie
            debug!("COOKIE struct: {}", cookie)
            match cookie.json.find(&"count".to_string()) {
                // Grab the string from the json's `count` key
                Some(&U64(cnt)) => {
                    // Increment our cookie counter
                    let cnt = cnt + 1;
                    let count = cnt.to_string();
                    println!("COOKIE COUNT: {}", count)
                    // Override the cookie with a new value
                    let mut res = Response::with(status::Ok, format!("Hit Counter: {}", count).as_slice());
                    res.set_json_cookie(cookie, ("count".to_string(), U64(cnt)), options);
                    return Ok(res);
                },
                _ => {
                    // Initialize our cookie counter
                    let mut res = Response::with(status::Ok, format!("Hit Counter: {}", 1u).as_slice());
                    res.set_json_cookie(cookie, ("count".to_string(), U64(1)), options);
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
