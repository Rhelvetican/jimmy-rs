#![doc = include_str!("../README.md")]

use std::io::Error;

pub type JimmyResult<T> = ::std::io::Result<T>;

mod jimmy;
mod state;

pub use jimmy::Jimmy;
pub use state::*;

#[cfg(test)]
mod tests {
    use crate::Jimmy;
    use std::io::{Error, Result};

    #[test]
    fn basic() {
        fn inner() -> Result<String> {
            let buf = Jimmy::new()?.field("eggs")?.array()?.object()?.field("amount")?.number(3.)?.end_object()?.end_array()?.finish()?;
            String::from_utf8(buf).map_err(|_| Error::other("utf8 error"))
        }

        assert_eq!(inner().unwrap(), "{\"eggs\":[{\"amount\":3}]}");
    }

    #[test]
    fn advanced() {
        fn inner() -> Result<String> {
            let buf = Jimmy::new()?
                .field("cord")?
                .object()?
                .field("inner_cord")?
                .object()?
                .field("inner")?
                .array()?
                .object()?
                .field("hello")?
                .boolean(true)?
                .end_object()?
                .end_array()?
                .end_object()?
                .end_object()?
                .finish()?;
            String::from_utf8(buf).map_err(|_| Error::other("utf8 error"))
        }

        assert_eq!(inner().unwrap(), "{\"cord\":{\"inner_cord\":{\"inner\":[{\"hello\":true}]}}}")
    }
}
