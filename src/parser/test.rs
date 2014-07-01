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
