use std::any::Any;

pub trait CastAny  {
    fn as_any_mut(&mut self) -> &mut dyn Any ;
    fn as_any(& self) -> & dyn Any ;
}

extern crate  macros_derive;
pub use macros_derive::CastAny;
pub use macros_derive::Node;
