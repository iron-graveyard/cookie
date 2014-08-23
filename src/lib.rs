#![crate_name = "cookie"]
#![deny(missing_doc)]
#![feature(phase)]
#![feature(globs)]

//! Cookie parsing/setting middleware for the [iron](https://github.com/iron/iron) framework.

extern crate time;
extern crate rustc;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
extern crate url;
extern crate serialize;
extern crate iron;
extern crate http;
extern crate crypto = "rust-crypto";
#[cfg(test)]
extern crate test = "iron-test";

pub use cookie::Cookie;
pub use parser::CookieParser;
pub use response::SetCookie;
pub use response::HeaderCollection;

mod parser;
mod response;
mod cookie;
