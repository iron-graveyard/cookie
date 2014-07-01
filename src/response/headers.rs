//! Cookie headers, as defined in RFC 6265

use time::Tm;
use std::collections::TreeMap;

/// The headers used to set a cookie.
///
/// These headers are defined by [RFC 6265](http://tools.ietf.org/html/rfc6265)
pub struct HeaderCollection {
    /// An absolute date/time at which this cookie should expire.
    pub expires:    Option<Tm>,
    /// A relative time (in seconds) at which this cookie should expire.
    pub max_age:    Option<u32>,
    /// The scope of the cookie.
    ///
    /// If set, the browser will send this cookie to the set domain and all subdomains.
    /// If not set, the browser will only send this cookie to the originating domain.
    ///
    /// This may only be set to the sending domain and its subdomains.
    pub domain:     Option<String>,
    /// The scope of the cookie.
    pub path:       Option<String>,
    /// A cookie with this flag should only be sent over secured/encrypted connections.
    ///
    /// This will be respected by the browser.
    pub secure:     bool,
    /// A cookie with this flag is only accessible through HTTP and HTTPS.
    ///
    /// This helps to prevent Javascript and, specifically, XSS attacks.
    pub http_only:  bool,
    /// Any additional headers.
    ///
    /// This may be any sequence of valid characters.
    ///
    /// Extensions will be separated with `;`.
    /// If a value is specified in the `Map`, the extension will be
    /// written as `[key]=[value]`.
    pub extensions: Option<TreeMap<String, Option<String>>>
}

impl HeaderCollection {
    #[doc(hidden)]
    pub fn to_cookie_av(self) -> String {
        let mut options = String::new()
            .append(head("Expires", self.expires, |v| v.rfc822()).as_slice())
            .append(head("Max-Age", self.max_age, |v| v.to_str()).as_slice())
            .append(head("Domain", self.domain, |v| v).as_slice())
            .append(head("Path", self.path, |v| v).as_slice());
        if self.secure { options.push_str("; Secure"); }
        if self.http_only { options.push_str("; Http-Only"); }
        match self.extensions {
            Some(map) => {
                for (header, value) in map.iter() {
                    options.push_str(extension(header, value.clone()).as_slice());
                }
            },
            None      => ()
        }
        options
    }
}

impl HeaderCollection {
    /// Convenience function for a set of empty cookie headers
    pub fn empty() -> HeaderCollection {
        HeaderCollection {
            expires: None,
            max_age: None,
            domain: None,
            path: None, 
            secure: false,
            http_only: false,
            extensions: None
        }
    }

    /// Convenience function for a set of cookie headers
    /// that will expire the cookie in `seconds` seconds
    pub fn aged(seconds: u32) -> HeaderCollection {
        HeaderCollection {
            expires: None,
            max_age: Some(seconds),
            domain: None,
            path: None, 
            secure: false,
            http_only: false,
            extensions: None
        }
    }

    /// Convenience function for a set of cookie headers
    /// declaring the cookie `Secure` and `HttpOnly`
    pub fn secured() -> HeaderCollection {
        HeaderCollection {
            expires: None,
            max_age: None,
            domain: None,
            path: None, 
            secure: true,
            http_only: true,
            extensions: None
        }
    }
}

fn head<V>(header: &str, value: Option<V>, mutator: |V| -> String) -> String {
    match value {
        Some(val) => {
            // Delimit from previous cookie/options
            "; ".to_string()
            // Add the header
                .append(header).append("=")
            // Add the mutated value
                .append(mutator(val).as_slice())
        },
        None      => String::new()
    }
}

fn extension(header: &String, value: Option<String>) -> String {
    match value {
        Some(val) => head(header.as_slice(), Some(val), |v| v),
        None      => "; ".to_string().append(header.as_slice())
    }
}
