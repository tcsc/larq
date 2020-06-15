// traits and types for abstracting away storage mechanisms

mod key;
mod store;

pub use key::{
    Key
};

pub use store::{
    Error,
    Include,
    ObjectInfo,
    Result,
    Store,
};