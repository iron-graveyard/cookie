//! Setting functionality - set cookie data

use std::collections::HashMap;
use iron::{Response};

/// Set cookies.
///
/// This trait is added as a mix-in to `Response`, allowing
/// simple cookie-setting.
pub trait SetCookie {
    /// Set cookies.
    ///
    /// Set cookies directly on the response with `res.set_cookie("coo=kie;")`.
    fn set_cookie(&mut self, (String, String), &HashMap<String, Option<String>>);
}

impl<'a> SetCookie for Response<'a> {
    fn set_cookie(&mut self, cookie: (String, String), options: &HashMap<String, Option<String>>) {
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
        // These need to be here so the compiler can correclty determine options' lifetime
        opt.unshift("; ");
        opt.pop();
        match cookie {
            (token, value) => {
                println!("{} {}", token, value)
                self.headers.extensions.insert(from_str::<String>("Set-Cookie").unwrap(),
                    token.append("=").append(value.as_slice()).append(opt.concat().as_slice()));
            }
        }
    }
}
