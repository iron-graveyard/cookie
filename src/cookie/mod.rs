//! Parsing functionality - get cookie data

use rustc::util::sha2::{Sha256, Digest};
use std::collections::hashmap::HashMap;
use serialize::json::{Json, Null};

/// The parsed cookie.
///
/// This is the type stored in the alloy.
#[deriving(Show)]
pub struct Cookie {
    secret: Option<String>,
    /// The parsed RFC 6265-styled cookies.
    pub map: HashMap<String, String>,
    /// Any JSON fields, parsed into a single object
    ///
    /// JSON stored under key `myJson` will be available
    /// under `cookie.json.find(&"myJson".to_string())`.
    pub json: Json
}

impl Cookie {
    /// Create a new cookie
    pub fn new(secret: Option<String>) -> Cookie {
        Cookie {
            secret: secret,
            map: HashMap::new(),
            json: Null
        }
    }

    /// Encode your signature
    ///
    /// Signatures will be encoded with SHA-256.
    pub fn sign(&self, value: &String) -> Option<String> {
        match self.secret {
            Some(ref secret) => {
                let mut sha = Sha256::new();
                sha.input_str(secret.as_slice());
                sha.input_str(value.as_slice());

                let hash = sha.result_str();

                // Purge hash of forbidden characters
                let mut signature = vec![];
                for &c in hash.as_bytes().iter() {
                    match c {
                        b'$'|b'='|b';'|b' '|b'\r'|b'\t'|b'\n' => (),
                        _ => signature.push(c)
                    }
                }

                // Return the encoded signature, if successful/available
                match ::std::str::from_utf8(signature.as_slice()) {
                    Some(sig) => Some(sig.to_string()),
                    None      => None
                }
            },
            None             => None
        }
    }
}
