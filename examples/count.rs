extern crate http;
extern crate iron;
extern crate cookie;
extern crate persistent;

use std::io::net::ip::Ipv4Addr;
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
            match cookie.map.find(&"count".to_string()) {
                Some(i) => {
                    // Convert the string to an int
                    let cnt: Option<int> = from_str(i.as_slice());
                    // Increment our cookie counter
                    let count = (cnt.unwrap() + 1).to_string();
                    println!("COOKIE COUNT: {}", i)
                    // Override the cookie with a new value
                    let mut res = Response::with(status::Ok, format!("Hit Counter: {}", count).as_slice());
                    res.set_cookie(&cookie, ("count".to_string(), count.clone()), options);
                    return Ok(res);
                },
                _ => {
                    // Initialize our cookie counter
                    let mut res = Response::with(status::Ok, format!("Hit Counter: {}", 1u8).as_slice());
                    res.set_cookie(&cookie, ("count".to_string(), "1".to_string()), options);
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
