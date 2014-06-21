//! Setting functionality - set cookie data

use std::collections::HashMap;
use iron::Response;
use super::Cookie;

/// Set cookies.
///
/// This trait is added as a mix-in to `Response`, allowing
/// simple cookie-setting.
pub trait SetCookie {
    /// Set cookies.
    ///
    /// Set cookies directly on the response with `res.set_cookie("coo=kie;")`.
    /// Only one cookie may sent per response, with the given key/value.
    /// Doing otherwise will result in ***undefined behavior***.
    ///
    /// Keys/values may not contain restricted characters:
    ///     `$`, `=`, `;`, and whitespace
    ///
    /// Cookies ***must*** be set before the response body is sent.
    /// Headers are flushed as soon anything is sent in the response body.
    fn set_cookie(&mut self, &Cookie, (String, String), &HashMap<String, Option<String>>);
}

impl<'a> SetCookie for Response<'a> {
    fn set_cookie(&mut self, signer: &Cookie, (key, value): (String, String), options: &HashMap<String, Option<String>>) {
        let mut opt = vec![]; // Avoid worrying about String moves
        for (key, value) in options.iter() {
            opt.push(key.as_slice());
            match *value {
                Some(ref val) => {
                    opt.push("=");
                    opt.push(val.as_slice());
                },
                None => (),
            }
            opt.push("; ");
        }
        // These need to be here so the compiler can correctly determine (key, value)'s lifetime
        opt.unshift("; ");
        opt.pop();

        self.headers.extensions.insert("Set-Cookie".to_string(),
            match signer.sign(&value) {
                Some(signature) => {
                    key.append("=")
                        .append("s:")
                        .append(value.as_slice())
                        .append(".")
                        .append(signature.as_slice())
                },
                None            => {
                    key.append("=")
                        .append(value.as_slice())
                }
            }.append(opt.concat().as_slice())
        );
    }
}
