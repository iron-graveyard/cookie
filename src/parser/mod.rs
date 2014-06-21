//! Parsing functionality - get cookie data

use std::collections::hashmap::HashMap;
use iron::{Request, Response, Middleware, Alloy};
use iron::middleware::{Status, Continue};

/// The cookie parsing `Middleware`.
///
/// It will parse the body of a cookie into the alloy, under type `Cookie`.
///
/// This middleware should be linked (added to the `Chain`)
/// before any other middleware using cookies, or the parsed cookie
/// will not be available to that middleware.
#[deriving(Clone)]
pub struct CookieParser;

/// The parsed cookie.
///
/// This is the type stored in the alloy.
#[deriving(Show)]
pub struct Cookie{
    /// The parsed RFC 6265-styled cookies.
    pub map: HashMap<String, Option<String>>,
    // json: Json TODO
    // options:   TODO
}

impl CookieParser {
    /// Create a new instance of the cookie parsing `Middleware`.
    ///
    /// This instance will parse both RFC 6265-styled cookies:
    /// `key=value; key=value;`
    /// and json-styled cookies, as set with `res.set_json_cookie(...)`.
    pub fn new() -> CookieParser { CookieParser }
}

impl Middleware for CookieParser {
    /// Parse the cookie received in the HTTP header.
    ///
    /// This will parse the body of a cookie into the alloy, under type `Cookie`.
    fn enter(&mut self, req: &mut Request, _res: &mut Response, alloy: &mut Alloy) -> Status {
        match req.headers.extensions.find_mut(&from_str::<String>("Cookie").unwrap()) {
            Some(cookie) => {
                let map: HashMap<String, Option<String>> =
                    cookie.as_slice().split(';').map(|substr| {
                        let vec: Vec<&str> = substr.splitn('=', 1).collect();
                        (if vec.get(0)[0] == b' ' { vec.get(0).slice_from(1).to_string() }
                            else { vec.get(0).to_string() },
                         if vec.len() == 1 { None } else { Some(vec.get(1).to_string()) })
                    }).collect();

                for (token, value) in map.iter() {
                    println!("{}: {}", token, value)
                }
                alloy.insert(Cookie{ map: map });
            },
            None => ()
        }
        Continue
    }
}
