//! Parsing functionality - get cookie data

use std::collections::treemap::TreeMap;
use url;
use serialize::json;
use serialize::json::{Json, Null};
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
        // Initialize a cookie. This will store parsed cookies and generate signatures.
        let mut new_cookie = Cookie::new(self.secret.clone());

        match req.headers.extensions.find_mut(&"Cookie".to_string()) {
            Some(cookies) => {
                // Initialize an empty json object.
                let mut new_json = json::Object(box TreeMap::new());
                new_cookie.map =
                    cookies
                        .as_slice()
                        .split(';')
                        // Decode from uri component encoding
                        .map(|substr| {
                            let vec: Vec<&str> = substr.splitn('=', 1).collect();
                            let key = from_rfc_compliant(*vec.get(0));
                            let val = from_rfc_compliant(*vec.get(1));
                            (key, val) })
                        // Check for signed cookies, and filter those not signed by us
                        .filter_map(|cookie| strip_signature(cookie, &new_cookie))
                        // Move json cookies into a separate container
                        .filter(|cookie| parse_json(cookie, &mut new_json))
                        .collect();

                // This cannot be inserted via iterators because strip_signature
                // is already borrowing new_cookie.
                new_cookie.json = new_json;
            },
            None => ()
        }
        alloy.insert(new_cookie);
        Continue
    }
}

fn from_rfc_compliant(string: &str) -> String {
    url::decode_component(
        string
            .chars()
            .skip_while(is_whitespace)
            .collect::<String>().as_slice())
}

fn is_whitespace(c: &char) -> bool {
    match *c {
        ' '|'\r'|'\t'|'\n' => true,
        _                  => false
    }
}

fn strip_signature((key, val): (String, String), signer: &Cookie) -> Option<(String, String)> {
    if val.len() > 2 && val.as_slice().slice(0, 2) == "s:" {
        // Extract the signature (in hex), appended onto the cookie after `.`
        return regex!(r"\.[^\.]*$").find(val.as_slice())
            // If it was signed by us, clear the signature
            .and_then(|(beg, end)| {
                signer.sign(&val.as_slice().slice(2, beg).to_string())
                    // We need to maintain access to (beg, end), so we chain the signature
                    .and_then(|signature| {
                        // If the signature is valid, strip it
                        if val.as_slice().slice(beg + 1, end) == signature.as_slice() {
                            // key must be cloned to move out of the closure capture
                            Some((key.clone(), val.as_slice().slice(2, beg).to_string()))
                        // Else, remove the cookie
                        } else {
                            None
                        }
                    })
            })
    }
    Some((key, val))
}

fn parse_json(&(ref key, ref val): &(String, String), json: &mut Json) -> bool {
    if val.len() > 2 && val.as_slice().slice(0, 2) == "j:" {
        match *json {
            json::Object(ref mut root) => {
                root.insert(key.clone(),
                    match json::from_str(val.as_slice().slice_from(2)) {
                        Ok(obj) => obj,
                        Err(_)  => Null
                    });
            },
            _                    => ()
        }
        return false
    }
    true
}

#[cfg(test)]
mod test {
    use std::mem::uninitialized;
    use std::collections::{HashMap, TreeMap};
    use http::headers::request::HeaderCollection;
    use iron::{Request, Alloy, Middleware};
    use super::*;
    use super::super::cookie::*;
    use serialize::json::{Object, String};

    // Parse a given `String` as an HTTP Cookie header, using the CookieParser middleware,
    // and return the cookie stored in the alloy by that middleware
    fn get_cookie<'a>(secret: Option<String>, cookie: String, alloy: &'a mut Alloy) -> &'a Cookie {
        let mut res = unsafe { Request{
            remote_addr: None,
            headers: box HeaderCollection::new(),
            body: "".to_string(),
            method: ::http::method::Get,
            request_uri: uninitialized(),
            close_connection: false,
            version: (1, 1)
        } };
        res.headers.extensions.insert("Cookie".to_string(), cookie);
        let mut signer = match secret {
            Some(s) => CookieParser::signed(s),
            None => CookieParser::new()
        };
        unsafe { signer.enter(&mut res, uninitialized(), alloy) };
        alloy.find::<Cookie>().unwrap()
    }

    #[test]
    fn check_cookie() {
        let mut alloy = Alloy::new();
        let cookie = get_cookie(None, "thing=thing".to_string(), &mut alloy);
        let mut map = HashMap::new();
        map.insert("thing".to_string(), "thing".to_string());
        assert_eq!(cookie.map, map);
    }

    #[test]
    fn check_escaping() {
        // Url component decoding should decode the escaped characters
        let mut alloy = Alloy::new();
        let cookie = get_cookie(None,
                                "~%60%21%40%23%24%25%5E%26%2A%28%29_%2B-%3D%7B%7D%7C%5B%5D%5C%3A%22%3B%27%3C%3E%3F%2C.%2F%27=\
                                ~%60%21%40%23%24%25%5E%26%2A%28%29_%2B-%3D%7B%7D%7C%5B%5D%5C%3A%22%3B%27%3C%3E%3F%2C.%2F%27".to_string(),
                                &mut alloy);
        let mut map = HashMap::new();
        map.insert("~`!@#$%^&*()_+-={}|[]\\:\";'<>?,./'".to_string(),
                   "~`!@#$%^&*()_+-={}|[]\\:\";'<>?,./'".to_string());
        assert_eq!(cookie.map, map);
    }

    #[test]
    fn check_signature() {
        // The signature should be a SHA-256 hash of @zzmpthung
        let mut alloy = Alloy::new();
        let cookie = get_cookie(Some("@zzmp".to_string()),
                                "thing=s:thung.2bc9a8b82a4a393ab67b2b8aaff0e3ab33cb4aca05ef4a0ba201141fbb029f42".to_string(),
                                &mut alloy);
        let mut map = HashMap::new();
        map.insert("thing".to_string(),
                   "thung".to_string());
        assert_eq!(cookie.map, map);
    }

    #[test]
    fn check_json() {
        // Parse the Url component JSON: {"thing":{"foo":"bar"}}
        let mut alloy = Alloy::new();
        let cookie = get_cookie(None,
                                "thing=j%3A%7B%22foo%22%3A%22bar%22%7D".to_string(),
                                &mut alloy);
        let mut child_map = TreeMap::new();
        child_map.insert("foo".to_string(), String("bar".to_string()));
        let child = Object(box child_map);
        let mut root_map = TreeMap::new();
        root_map.insert("thing".to_string(), child);
        let root = Object(box root_map);
        assert_eq!(cookie.json, root); // FIXME
    }
}
