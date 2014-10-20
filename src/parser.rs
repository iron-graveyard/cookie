//! Parsing functionality - get cookie data

use std::collections::treemap::TreeMap;
use url::lossy_utf8_percent_decode;
use serialize::json;
use serialize::json::{Json, Null};
use iron::{Request, IronResult, Plugin};
use iron::typemap::Assoc;
use super::Cookie;
use crypto::util::fixed_time_eq;
use plugin::{PluginFor, Phantom};
use persistent::Read;

/// The cookie parsing `Middleware`.
///
/// It will parse the body of a cookie into the alloy, under type `Cookie`.
///
/// This middleware should be linked (added to the `Chain`)
/// before any other middleware using cookies, or the parsed cookie
/// will not be available to that middleware.
#[deriving(Clone)]
pub struct CookieSettings {
    pub secret: Option<String>,
}

pub struct CookieParser;

impl Assoc<CookieSettings> for CookieParser {}

impl Assoc<Cookie> for CookieParser {}

impl PluginFor<Request, Cookie> for CookieParser {
    /// Parse the cookie received in the HTTP header.
    ///
    /// This will parse the body of a cookie into the alloy, under type `Cookie`.
    fn eval(req: &Request, _: Phantom<CookieParser>) -> Option<Cookie> {
        let CookieSettings { secret }: CookieSettings
            = req.get::<Read<CookieParser, CookieSettings>>().unwrap();
        let mut new_cookie = Cookie::new(secret.clone());

        match req.headers.extensions.find_mut(&"Cookie".to_string()) {
            Some(cookies) => {
                //Initialize an empty json object.
                let mut new_json = json::Object(TreeMap::new());
                new_cookie.map =
                    cookies
                        .as_slice()
                        .split(';')
                        // Decode from uri component encoding
                        .map(|substr| {
                            let vec: Vec<&str> = substr.splitn(1, '=').collect();
                            let key = from_rfc_compliant(vec[0]);
                            let val = from_rfc_compliant(vec[1]);
                            (key, val) })
                        // Check for signed cookies, and filter those not signed by us
                        .filter_map(|cookie| strip_signature(cookie, &new_cookie))
                        // Move json cookies into a separate container
                        .filter(|cookie| parse_json(cookie, &mut new_json))
                        .collect();

                // This cannot be inserted via iterators because strip_signature
                // is already borrowing new_cookie.
                new_cookie.json = new_json;
                Some(new_cookie)
            },
            None => None
        }
    }
}

fn from_rfc_compliant(string: &str) -> String {
    lossy_utf8_percent_decode(
        string
            .chars()
            .skip_while(is_whitespace)
            .collect::<String>()
            .as_bytes()
    )
}

fn is_whitespace(c: &char) -> bool {
    match *c {
        ' '|'\r'|'\t'|'\n' => true,
        _                  => false
    }
}

fn strip_signature((key, val): (String, String), signer: &Cookie) -> Option<(String, String)> {
    if val.len() > 2 && val.as_slice().slice(0, 2) == "s:" {
        if !signer.signed { return None }
        // Extract the signature (in hex), appended onto the cookie after `.`
        return regex!(r"\.[^\.]*$").find(val.as_slice())
            // If it was signed by us, clear the signature
            .and_then(|(beg, end)| {
                signer.sign(&val.as_slice().slice(2, beg).to_string())
                    // We need to maintain access to (beg, end), so we chain the signature
                    .and_then(|signature| {
                        // If the signature is valid, strip it
                         if fixed_time_eq(val.as_slice().slice(beg + 1, end).as_bytes(), signature.as_bytes()) {
                            // key must be cloned to move out of the closure capture
                            Some((key.clone(), val.as_slice().slice(2, beg).to_string()))
                        // Else, remove the cookie
                        } else {
                            None
                        }
                    })
            })
    }
    match signer.signed {
        true => None,
        false => Some((key, val))
    }
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
    use std::collections::{HashMap, TreeMap};
    use iron::{Request, BeforeMiddleware};
    use test::mock::request;
    use super::*;
    use super::super::cookie::*;
    use serialize::json::{Object, String};

    // Parse a given `String` as an HTTP Cookie header, using the CookieParser middleware,
    // and return the cookie stored in the alloy by that middleware
    fn get_cookie_request(secret: Option<String>, cookie: String) -> Request {
        let mut req = request::new(::http::method::Get, "localhost:3000");
        req.headers.extensions.insert("Cookie".to_string(), cookie);
        let signer = match secret {
            Some(s) => CookieParser::signed(s),
            None => CookieParser::new()
        };
        let _ = signer.before(&mut req);
        req
    }

    #[test]
    fn check_cookie() {
        let cookie_request = get_cookie_request(None, "thing=thing".to_string());
        let cookie = cookie_request.extensions.find::<CookieParser, Cookie>().unwrap();
        let mut map = HashMap::new();
        map.insert("thing".to_string(), "thing".to_string());
        assert_eq!(cookie.map, map);
    }

    #[test]
    fn check_escaping() {
        // Url component decoding should decode the escaped characters
        let cookie_request = get_cookie_request(None,
                                "~%60%21%40%23%24%25%5E%26%2A%28%29_%2B-%3D%7B%7D%7C%5B%5D%5C%3A%22%3B%27%3C%3E%3F%2C.%2F%27=\
                                ~%60%21%40%23%24%25%5E%26%2A%28%29_%2B-%3D%7B%7D%7C%5B%5D%5C%3A%22%3B%27%3C%3E%3F%2C.%2F%27".to_string());
        let cookie = cookie_request.extensions.find::<CookieParser, Cookie>().unwrap();
        let mut map = HashMap::new();
        map.insert("~`!@#$%^&*()_+-={}|[]\\:\";'<>?,./'".to_string(),
                   "~`!@#$%^&*()_+-={}|[]\\:\";'<>?,./'".to_string());
        assert_eq!(cookie.map, map);
    }

    #[test]
    fn check_signature() {
        // The signature should be the HMAC-SHA256 hash of key "@zzmp" and message "thung"
        let cookie_request = get_cookie_request(Some("@zzmp".to_string()),
                                "thing=s:thung.e99abddcf60cad18f8d4b993efae53e81410cf2b2855af0309f1ae46fa527fbb".to_string());
        let cookie = cookie_request.extensions.find::<CookieParser, Cookie>().unwrap();
        let mut map = HashMap::new();
        map.insert("thing".to_string(),
                   "thung".to_string());
        assert_eq!(cookie.map, map);
    }

    #[test]
    fn check_silo() {
        // The unsigned cookie should not be parsed by the signed cookie parser
        let cookie_request = get_cookie_request(Some("@zzmp".to_string()),
                                "thing=thung".to_string());
        let cookie = cookie_request.extensions.find::<CookieParser, Cookie>().unwrap();
        let map = HashMap::new();
        assert_eq!(cookie.map, map);
    }

    #[test]
    fn check_json() {
        // Parse the Url component JSON: {"thing":{"foo":"bar"}}
        let cookie_request = get_cookie_request(None,
                                "thing=j%3A%7B%22foo%22%3A%22bar%22%7D".to_string());
        let cookie = cookie_request.extensions.find::<CookieParser, Cookie>().unwrap();
        let mut child_map = TreeMap::new();
        child_map.insert("foo".to_string(), String("bar".to_string()));
        let child = Object(child_map);
        let mut root_map = TreeMap::new();
        root_map.insert("thing".to_string(), child);
        let root = Object(root_map);
        assert_eq!(cookie.json, root); // FIXME
    }
}
