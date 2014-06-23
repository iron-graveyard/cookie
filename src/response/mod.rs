//! Setting functionality - set cookie data

use url;
use serialize::json::{Json, Number, String, Boolean, List, Object, Null};
use iron::Response;
use super::Cookie;
use self::headers::HeaderCollection;

pub mod headers;

/// Set cookies.
///
/// This trait is added as a mix-in to `Response`, allowing
/// simple cookie-setting.
pub trait SetCookie {
    /// Set a cookie.
    ///
    /// Set cookies directly on the response with `res.set_cookie("coo=kie;")`.
    /// Only one cookie may sent per response, with the given key/value.
    /// Doing otherwise will result in ***undefined behavior***.
    ///
    /// Keys/values may contain restricted characters, but they will be URI encoded in the cookie.
    ///
    /// They will be decoded when the cookie is returned to the server.
    ///
    /// Cookies ***must*** be set before the response body is sent.
    /// Headers are flushed as soon anything is sent in the response body.
    fn set_cookie(&mut self, &Cookie, (String, String), HeaderCollection);

    /// Set a cookie as JSON.
    ///
    /// Cookies set as JSON will be available under `cookie.json`.
    /// Otherwise, they behave exactly as normally serialized cookies.
    ///
    /// Note that restricted characters will still be URI encoded in your cookie.
    ///
    /// They will be decoded when the cookie is returned to the server.
    fn set_json_cookie(&mut self, &Cookie, (String, Json), HeaderCollection);
}

impl<'a> SetCookie for Response<'a> {
    fn set_cookie(&mut self,
                  signer: &Cookie,
                  (key, value): (String, String),
                  options: HeaderCollection) {

        self.headers.extensions.insert("Set-Cookie".to_string(),
            match signer.sign(&value) {
                Some(signature) => {
                    url::encode_component(key.as_slice())
                        .append("=")
                        .append("s:")
                        .append(url::encode_component(value.as_slice()).as_slice())
                        .append(".")
                        .append(signature.as_slice())
                },
                None            => {
                    url::encode_component(key.as_slice())
                        .append("=")
                        .append(url::encode_component(value.as_slice()).as_slice())
                }
            }.append(options.to_cookie_av().as_slice())
        );
    }

    fn set_json_cookie(&mut self,
                       signer: &Cookie,
                       (key, value): (String, Json),
                       options: HeaderCollection) {
        let json = "j:".to_string().append(stringify_json(&value).as_slice());
        self.set_cookie(signer, (key, json), options)
    }
}

fn stringify_json(json: &Json) -> String {
    match *json {
        Object(ref object) => {
            let obj: Vec<String> = object.iter().map(stringify_pair).collect();
            "{".to_string().append(obj.connect(",").as_slice()).append("}")
        },
        List(ref list)     => {
            let ary: Vec<String> = list.iter().map(stringify_json).collect();
            "[".to_string().append(ary.connect(",").as_slice()).append("]")
        },
        Number(number) => number.to_str(),
        String(ref string) => "\"".to_string().append(string.as_slice()).append("\""),
        Boolean(true)      => "true".to_string(),
        Boolean(false)     => "false".to_string(),
        Null               => "null".to_string()
    }
}

fn stringify_pair((key, val): (&String, &Json)) -> String {
    "\"".to_string().append(key.as_slice()).append("\":").append(stringify_json(val).as_slice())
}
