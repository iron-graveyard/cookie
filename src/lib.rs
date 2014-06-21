#![crate_id = "cookie"]
#![deny(missing_doc)]

//! Cookie parsing/setting middleware for the [iron](https://github.com/iron/iron) framework.

extern crate iron;
extern crate http;

pub use parser::{CookieParser, Cookie};
pub use response::SetCookie;

pub mod parser;
pub mod response;
