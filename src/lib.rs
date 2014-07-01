#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/cookie")]
#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/cookie")]
#![crate_id = "cookie"]
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

pub use parser::CookieParser;
pub use response::SetCookie;
pub use cookie::Cookie;
pub use response::headers::HeaderCollection;

pub mod parser;
pub mod response;
pub mod cookie;
