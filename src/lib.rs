#![crate_id = "cookie"]
#![deny(missing_doc)]
#![feature(phase)]

//! Cookie parsing/setting middleware for the [iron](https://github.com/iron/iron) framework.

extern crate rustc;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
extern crate serialize;
extern crate iron;
extern crate http;

pub use parser::CookieParser;
pub use response::SetCookie;
pub use cookie::Cookie;

pub mod parser;
pub mod response;
pub mod cookie;
