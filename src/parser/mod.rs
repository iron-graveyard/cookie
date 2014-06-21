//! Parsing functionality - get cookie data

use std::collections::hashmap::HashMap;
use iron::{Request, Response, Middleware, Alloy};
use iron::middleware::{Status, Continue};
use super::Cookie;

/// The cookie parsing `Middleware`.
///
/// It will parse the body of a cookie into the alloy, under type `Cookie`.
///
/// This middleware should be linked (added to the `Chain`)
/// before any other middleware using cookies, or the parsed cookie
/// will not be available to that middleware.
#[deriving(Clone)]
pub struct CookieParser {
    secret: Option<String>
}

impl CookieParser {
    /// Create a new instance of the cookie parsing `Middleware`.
    ///
    /// This instance will parse both RFC 6265-styled cookies:
    /// `key=value; key=value;`
    /// and json-styled cookies, as set with `res.set_json_cookie(...)`.
    pub fn new() -> CookieParser { CookieParser{ secret: None} }

    /// Create a cookie parser with secret, for signed cookies.
    ///
    /// This instance will parse any cookies that have been signed by
    /// you, or that are unsigned. It will not parse those cookies signed by others.
    ///
    /// Otherwise, it will behave exactly like that produced by `new`.
    pub fn signed(secret: String) -> CookieParser { CookieParser{ secret: Some(secret) } }
}

impl Middleware for CookieParser {
    /// Parse the cookie received in the HTTP header.
    ///
    /// This will parse the body of a cookie into the alloy, under type `Cookie`.
    fn enter(&mut self, req: &mut Request, _res: &mut Response, alloy: &mut Alloy) -> Status {
        let mut parsed_cookie = Cookie::new(self.secret.clone());
        match req.headers.extensions.find_mut(&"Cookie".to_string()) {
            Some(cookie) => {
                let mut map: HashMap<String, String> =
                    cookie.as_slice().split(';').map(|substr| {
                        let vec: Vec<&str> = substr.splitn('=', 1).collect();
                        (if vec.get(0)[0] == b' ' { vec.get(0).slice_from(1).to_string() }
                            else { vec.get(0).to_string() },
                         if vec.len() == 1 { "".to_string() } else { vec.get(1).to_string() })
                    }).collect();

                match self.secret {
                    Some(ref _secret) => {
                        let mut tokens = vec![];
                        for (token, value) in map.mut_iter() {
                            if value.len() > 2 && value.as_slice().slice(0, 2) == "s:" {
                                match regex!(r"\.[^\.]*$").find(value.as_slice()) {
                                    Some((beg, end)) => {
                                        // If it was signed by us, clear the signature
                                        match parsed_cookie.sign(&value.as_slice().slice(2, beg).to_string()) {
                                            Some(signature) => {
                                                if value.as_slice().slice(beg + 1, end) == signature.as_slice() {
                                                    *value = value.as_slice().slice(2, beg).to_string();
                                                // Else, set them for removal
                                                } else {
                                                    tokens.push(token.clone());    
                                                }
                                            },
                                            None            => {
                                                tokens.push(token.clone())
                                            }
                                        }
                                    },
                                    None           => {
                                    }
                                }
                            }
                        }
                        for token in tokens.iter() {
                            map.remove(token);
                        }
                    },
                    None         => ()
                }
                parsed_cookie.map = map;
                alloy.insert(parsed_cookie);
            },
            None => { alloy.insert(parsed_cookie); }
        }
        Continue
    }
}
