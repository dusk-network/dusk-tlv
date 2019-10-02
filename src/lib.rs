#![feature(external_doc)]
#![deny(missing_docs)]
#![doc(include = "../README.md")]

mod reader;
mod writer;

pub use reader::TlvReader;
pub use writer::TlvWriter;
