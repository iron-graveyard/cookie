use std::mem::uninitialized;
use std::collections::HashMap;
use http::headers::request::HeaderCollection;
use iron::{Request, Alloy, Middleware};
use super::*;
use super::super::cookie::*;
// use serialize::json::{Json, ToJson};

fn get_cookie<'a>(secret: Option<String>, cookie: &str, alloy: &'a mut Alloy) -> &'a Cookie {
    let mut res = unsafe{ Request{
        remote_addr: uninitialized(),
        headers: box HeaderCollection::new(),
        body: uninitialized(),
        method: uninitialized(),
        request_uri: uninitialized(),
        close_connection: uninitialized(),
        version: uninitialized()
    } };
    res.headers.extensions.insert("Cookie".to_string(), cookie.to_string());
    let mut signer = match secret {
        Some(s) => CookieParser::signed(s),
        None => CookieParser::new()
    };
    unsafe{ signer.enter(&mut res, uninitialized(), alloy) };
    alloy.find::<Cookie>().unwrap()
}

#[test]
fn check_cookie() {
    let mut alloy = Alloy::new();
    let cookie = get_cookie(None, "thing=thing", &mut alloy);
    let mut map = HashMap::new();
    map.insert("thing".to_string(), "thing".to_string());
    assert_eq!(cookie.map, map);
}

// #[test]
// fn check_escaping() {
//     let headers = HeaderCollection::empty();
//     assert_eq!(get_cookie(headers, None, "~`!@#$%^&*()_+-={}|[]\\:\";'<>?,./'", "~`!@#$%^&*()_+-={}|[]\\:\";'<>?,./'"),
//         // Url component encoding
//         "~%60%21%40%23%24%25%5E%26%2A%28%29_%2B-%3D%7B%7D%7C%5B%5D%5C%3A%22%3B%27%3C%3E%3F%2C.%2F%27=\
//          ~%60%21%40%23%24%25%5E%26%2A%28%29_%2B-%3D%7B%7D%7C%5B%5D%5C%3A%22%3B%27%3C%3E%3F%2C.%2F%27".to_string());
// }

// #[test]
// fn check_headers() {
//     let mut headers = HeaderCollection {
//         expires:    None,
//         max_age:    Some(42),
//         domain:     Some("example.com".to_string()),
//         path:       Some("/a/path".to_string()),
//         secure:     true,
//         http_only:  true,
//         extensions: Some(TreeMap::<String, Option<String>>::new())
//     };
//     headers.extensions.as_mut().unwrap().insert("foo".to_string(), Some("bar".to_string()));
//     headers.extensions.as_mut().unwrap().insert("@zzmp".to_string(), None);
//     assert_eq!(get_cookie(headers, None, "thing", "thing"),
//         "thing=thing; Max-Age=42; Domain=example.com; Path=/a/path; Secure; Http-Only; @zzmp; foo=bar".to_string());
// }

// #[test]
// fn check_signature() {
//     let headers = HeaderCollection::empty();
//     assert_eq!(get_cookie(headers, Some("@zzmp".to_string()), "thing", "thung"),
//         // Hash of @zzmpthung
//         "thing=s:thung.2bc9a8b82a4a393ab67b2b8aaff0e3ab33cb4aca05ef4a0ba201141fbb029f42".to_string());
// }

// //TO DO
// #[test]
// fn check_json() {
//     let headers = HeaderCollection::empty();
//     assert_eq!(get_json_cookie(headers, None, "thing", "{\"foo\":\"bar\"}".to_string().to_json()),
//         // Url component encoded
//         "thing=j%3A%22%7B%22foo%22%3A%22bar%22%7D%22".to_string());
// }