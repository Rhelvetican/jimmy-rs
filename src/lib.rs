//! # Jimmy
//!
//! A type-safe JSON builder using the typestate pattern.

use std::{
    io::{Error, Write},
    marker::PhantomData,
};

macro_rules! ret {
    ($sink:expr, $reentrant:expr) => {
        Ok(Jimmy {
            sink:      $sink,
            reentrant: $reentrant,
            __state:   ::std::marker::PhantomData,
        })
    };
}

macro_rules! defstates {
    ($($state:ident $(= $doc:literal)?),+) => {
        $(
            $(#[doc = $doc])?
            pub struct $state<Prev> {
                _prev: ::std::marker::PhantomData<Prev>,
            }
        )+
    };
}

macro_rules! implstates {
    ($next:ty) => {
        /// Write a JSON number.
        pub fn number(mut self, value: f64) -> Result<Jimmy<W, $next>, Error> {
            self.comma()?;
            write!(self.sink, "{value}")?;
            ret!(self.sink, true)
        }

        /// Write a JSON string.
        pub fn string(mut self, value: &str) -> Result<Jimmy<W, $next>, Error> {
            self.comma()?;
            write!(self.sink, "\"{value}\"")?;
            ret!(self.sink, true)
        }

        /// Write a JSON boolean.
        pub fn boolean(mut self, boolean: bool) -> Result<Jimmy<W, $next>, Error> {
            self.comma()?;
            write!(self.sink, "{boolean}")?;
            ret!(self.sink, true)
        }

        /// Write a JSON null.
        pub fn null(mut self) -> Result<Jimmy<W, $next>, Error> {
            self.comma()?;
            write!(self.sink, "null")?;
            ret!(self.sink, true)
        }

        /// Start a JSON array.
        pub fn array(mut self) -> Result<Jimmy<W, Array<$next>>, Error> {
            self.comma()?;
            write!(self.sink, "[")?;
            ret!(self.sink, false)
        }

        /// Start a JSON object.
        pub fn object(mut self) -> Result<Jimmy<W, Object<$next>>, Error> {
            self.comma()?;
            write!(self.sink, "{{")?;
            ret!(self.sink, false)
        }
    };
}

/// Root state marker.
pub struct Root;
defstates!(Array = "Array state marker.", Object = "Object state marker.", Field = "Field state marker.");

/// The main JSON builder.
///
/// It holds the underlying writer (`W`) and a boolean `reentrant` that indicates
/// whether the next element needs a leading comma. The `__state` field is a
/// phantom type that encodes the current position in the JSON structure, ensuring
/// that only valid method calls are allowed at compile time.
#[derive(Debug)]
pub struct Jimmy<W, S> {
    /// Main sink.
    pub sink:  W,
    reentrant: bool,
    __state:   PhantomData<S>,
}

impl<W: Write> Jimmy<W, Root> {
    /// Start a new JSON object. The builder is initially in the root state.
    ///
    /// The opening brace `{` is written immediately.
    pub fn new(mut sink: W) -> Result<Self, Error> {
        write!(sink, "{{")?;
        ret!(sink, false)
    }

    /// Add a field name inside the root object.
    ///
    /// This method writes the field name followed by a colon and transitions to
    /// the `Field` state, where a value must be provided.
    pub fn field(mut self, field: &str) -> Result<Jimmy<W, Field<Root>>, Error> {
        self.comma()?;
        write!(self.sink, "\"{field}\":")?;
        ret!(self.sink, false)
    }

    /// Finish the JSON object and close the root.
    ///
    /// Writes the closing brace `}` and consumes the builder.
    pub fn finish(mut self) -> Result<W, Error> {
        write!(self.sink, "}}")?;
        self.sink.flush()?;
        Ok(self.sink)
    }
}

impl<W: Write, State> Jimmy<W, State> {
    /// Write a comma if the previous element requires one.
    ///
    /// This is used internally to insert separators between JSON elements.
    pub fn comma(&mut self) -> Result<(), Error> { if self.reentrant { write!(self.sink, ",") } else { Ok(()) } }
}

impl<W: Write, Prev> Jimmy<W, Field<Prev>> {
    implstates!(Prev);
}

impl<W: Write, Prev> Jimmy<W, Object<Prev>> {
    /// Add a field name inside the current object.
    ///
    /// Writes the field name and colon, and transitions to the `Field` state.
    pub fn field(mut self, field: &str) -> Result<Jimmy<W, Field<Object<Prev>>>, Error> {
        self.comma()?;
        write!(self.sink, "\"{field}\":")?;
        ret!(self.sink, false)
    }

    /// Close the current object and return to the parent state.
    ///
    /// Writes the closing brace `}`. After this, the parent state may continue
    /// with more elements or close further.
    pub fn end_object(mut self) -> Result<Jimmy<W, Prev>, Error> {
        write!(self.sink, "}}")?;
        ret!(self.sink, true)
    }
}

impl<W: Write, Prev> Jimmy<W, Array<Prev>> {
    implstates!(Array<Prev>);

    /// Close the current array and return to the parent state.
    ///
    /// Writes the closing bracket `]`. After this, the parent state may continue.
    pub fn end_array(mut self) -> Result<Jimmy<W, Prev>, Error> {
        write!(self.sink, "]")?;
        ret!(self.sink, true)
    }
}

#[cfg(test)]
mod tests {
    use crate::Jimmy;
    use std::io::{Error, Result};

    #[test]
    fn basic() {
        fn inner() -> Result<String> {
            let mut buf = Vec::new();
            Jimmy::new(&mut buf)?.field("eggs")?.array()?.object()?.field("amount")?.number(3.)?.end_object()?.end_array()?.finish()?;
            String::from_utf8(buf).map_err(|_| Error::other("utf8 error"))
        }

        assert_eq!(inner().unwrap(), "{\"eggs\":[{\"amount\":3}]}");
    }

    #[test]
    fn advanced() {
        fn inner() -> Result<String> {
            let mut buf = Vec::new();
            Jimmy::new(&mut buf)?
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
