macro_rules! defstates {
    ($($state:ident = $doc:literal),+) => {
        $(
            #[doc = $doc]
            #[derive(Debug, Clone, Copy)]
            pub struct $state<Prev> {
                __prev: ::std::marker::PhantomData<Prev>,
            }
        )+
    };
}

/// Root state marker.
#[derive(Clone, Copy)]
pub struct Root;

defstates!(Array = "Array state marker.", Object = "Object state marker.", Field = "Field state marker.");
