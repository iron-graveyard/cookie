#![feature(phase)]
#[phase(plugin, link)] extern crate log;
extern crate http;
extern crate iron;
extern crate cookie;
extern crate serialize;
extern crate persistent;

use std::io::net::ip::Ipv4Addr;
use serialize::json::U64;
use http::status;
use iron::{Iron, Plugin, ChainBuilder, Chain, Request, Response, IronResult};
use cookie::{CookieParser, CookieSettings, Cookie, SetCookie, HeaderCollection};
use persistent::Read;

fn count_views(req: &mut Request) -> IronResult<Response> {
    // Only hold on to cookies for ten seconds
    let options = HeaderCollection::aged(10);

    match req.get::<CookieParser>() {
        Some(cookie) => {
            // Find the "count" cookie
            debug!("COOKIE struct: {}", cookie)
            match cookie.json.find(&"count".to_string()) {
                // Grab the string from the json's `count` key
                Some(&U64(cnt)) => {
                    // Increment our cookie counter
                    let cnt = cnt + 1;
                    let count = cnt.to_string();
                    println!("COOKIE COUNT: {}", count);
                    // Override the cookie with a new value
                    let mut res = Response::with(status::Ok, format!("Hit Counter: {}", count).as_slice());
                    res.set_json_cookie(&cookie, ("count".to_string(), U64(cnt)), options);
                    return Ok(res);
                },
                _ => {
                    // Initialize our cookie counter
                    let mut res = Response::with(status::Ok, format!("Hit Counter: {}", 1u).as_slice());
                    res.set_json_cookie(&cookie, ("count".to_string(), U64(1)), options);
                    return Ok(res);
                }
            }
        },
        _ => {} // This should never occur, so long as the CookieSettings is linked first
    }
    Ok(Response::new())
}

fn main() {
    let mut chain = ChainBuilder::new(count_views);
    let cookie_settings = CookieSettings { secret: Some("@zzmp".to_string()) };
    chain.link(Read::<CookieParser, CookieSettings>::both(cookie_settings));

    Iron::new(chain).listen(Ipv4Addr(127, 0, 0, 1), 3000);
    println!("Server listening on 3000!");
}
