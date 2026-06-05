
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

mod karatsuba;
mod schoolbook;
mod squaring;

pub(crate) use karatsuba::*;
pub(crate) use schoolbook::*;
pub(crate) use squaring::*;
