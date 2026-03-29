use crate::*;
use std::{io::Write, marker::PhantomData};

macro_rules! ret {
    ($sink:expr, $reentrant:expr) => {
        Ok(Jimmy {
            sink:      $sink,
            reentrant: $reentrant,
            __state:   ::std::marker::PhantomData,
        })
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

/// The main JSON builder.
#[derive(Debug)]
pub struct Jimmy<W, S>
where
    W: Write,
{
    sink:      W,
    reentrant: bool,
    __state:   PhantomData<S>,
}

impl<W: Write, State> Jimmy<W, State> {
    /// Write a comma if the previous element requires one.
    ///
    /// This is used internally to insert separators between JSON elements.
    fn comma(&mut self) -> Result<(), Error> { if self.reentrant { write!(self.sink, ",") } else { Ok(()) } }
}

impl<W: Write, Prev> Jimmy<W, Field<Prev>> {
    implstates!(Prev);
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

impl<W: Write, Prev> Jimmy<W, Object<Prev>> {
    /// Add a field name inside the current object.
    ///
    /// Writes the field name and colon, and transitions to the `Field` state.
    pub fn field(mut self, field: &str) -> Result<Jimmy<W, Field<Object<Prev>>>, Error> {
        self.comma()?;
        write!(self.sink, "\"{}\":", field)?;
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

impl Jimmy<Vec<u8>, Root> {
    /// Start a new JSON object with an `Vec<u8>`. The builder is initially in the root state.
    ///
    /// The opening brace `{` is written immediately.
    #[inline]
    pub fn new() -> JimmyResult<Self> { Self::new_with_writer(Vec::new()) }

    /// Start a new JSON object with an `Vec<u8>` and a given capacity. The builder is initially in the root state.
    ///
    /// The opening brace `{` is written immediately.
    #[inline]
    pub fn new_with_capacity(capacity: usize) -> JimmyResult<Self> { Self::new_with_writer(Vec::with_capacity(capacity)) }
}

impl<W: Write> Jimmy<W, Root> {
    /// Start a new JSON object using the given writer. The builder is initially in the root state.
    ///
    /// The opening brace `{` is written immediately.
    #[inline]
    pub fn new_with_writer(mut writer: W) -> JimmyResult<Self> {
        write!(&mut writer, "{{")?;
        ret!(writer, false)
    }

    /// Add a field name inside the root object.
    ///
    /// This method writes the field name followed by a colon and transitions to
    /// the `Field` state, where a value must be provided.
    pub fn field(mut self, field: &str) -> JimmyResult<Jimmy<W, Field<Root>>> {
        self.comma()?;
        write!(self.sink, "\"{}\":", field)?;
        ret!(self.sink, false)
    }

    /// Finish the JSON object and close the root.
    ///
    /// Writes the closing brace `}` and consumes the builder.
    pub fn finish(mut self) -> JimmyResult<W> {
        write!(self.sink, "}}")?;
        self.sink.flush()?;
        Ok(self.sink)
    }
}
