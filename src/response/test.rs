pub use std::mem::uninitialized;
pub use std::collections::TreeMap;
pub use http::server::response::ResponseWriter;
pub use http::headers::response::HeaderCollection;
pub use iron::*;
pub use super::*;
pub use super::headers::*;
pub use super::super::cookie::*;
use serialize::json::Json;

pub fn get_cookie<'a>(headers: HeaderCollection, secret: Option<String>, key: &str, value: &str) -> String {
    let mut res = unsafe{ ResponseWriter::new(uninitialized()) };
    let signer = Cookie::new(secret);
    let cookie = (key.to_string(), value.to_string());
    res.set_cookie(&signer, cookie, headers);
    res.headers.extensions.find(&"Set-Cookie".to_string()).unwrap().clone()
}

pub fn get_json_cookie<'a>(headers: HeaderCollection, secret: Option<String>, key: &str, value: &Json) -> String {
    let mut res = unsafe{ ResponseWriter::new(uninitialized()) };
    let signer = Cookie::new(secret);
    let cookie = (key.to_string(), value.clone());
    res.set_json_cookie(&signer, cookie, headers);
    res.headers.extensions.find(&"Set-Cookie".to_string()).unwrap().clone()
}

#[test]
fn check_cookie() {
    let headers = HeaderCollection::empty();
    assert_eq!(get_cookie(headers, None, "thing", "thing"), "thing=thing".to_string());
}

#[test]
fn check_escaping() {
    let headers = HeaderCollection::empty();
    assert_eq!(get_cookie(headers, None, "~`!@#$%^&*()_+-={}|[]\\:\";'<>?,./'", "~`!@#$%^&*()_+-={}|[]\\:\";'<>?,./'"),
        "~%60%21%40%23%24%25%5E%26%2A%28%29_%2B-%3D%7B%7D%7C%5B%5D%5C%3A%22%3B%27%3C%3E%3F%2C.%2F%27=\
         ~%60%21%40%23%24%25%5E%26%2A%28%29_%2B-%3D%7B%7D%7C%5B%5D%5C%3A%22%3B%27%3C%3E%3F%2C.%2F%27".to_string());
}

#[test]
fn check_headers() {
    let mut headers = HeaderCollection {
        expires:    None,
        max_age:    Some(42),
        domain:     Some("example.com".to_string()),
        path:       Some("/a/path".to_string()),
        secure:     true,
        http_only:  true,
        extensions: Some(TreeMap::<String, Option<String>>::new())
    };
    headers.extensions.as_mut().unwrap().insert("foo".to_string(), Some("bar".to_string()));
    headers.extensions.as_mut().unwrap().insert("@zzmp".to_string(), None);
    assert_eq!(get_cookie(headers, None, "thing", "thing"),
        "thing=thing; Max-Age=42; Domain=example.com; Path=/a/path; Secure; Http-Only; @zzmp; foo=bar".to_string());
}

#[test]
fn check_signature() {
    let headers = HeaderCollection::empty();
    assert_eq!(get_cookie(headers, Some("@zzmp".to_string()), "thing", "thung"),
        // Hash of @zzmpthung
        "thing=s:thung.2bc9a8b82a4a393ab67b2b8aaff0e3ab33cb4aca05ef4a0ba201141fbb029f42".to_string());
}

//TO DO
#[test]
fn check_json() {

}

#[test]
fn check_signed_json() {

}
