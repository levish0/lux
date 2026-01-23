pub mod attribute;
pub mod comment;
pub mod element;
pub mod expression;
pub mod fragment;
pub mod tag;
pub mod text;

use winnow::stream::{LocatingSlice, Stateful};

use crate::context::ParseContext;

pub type InputSource<'i> = LocatingSlice<&'i str>;
pub type ParserInput<'i> = Stateful<InputSource<'i>, ParseContext>;
