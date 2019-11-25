#![feature(external_doc)]
#![deny(missing_docs)]
#![doc(include = "../README.md")]

mod error;
mod reader;
mod writer;

pub use error::Error;
pub use reader::TlvReader;
pub use writer::TlvWriter;
