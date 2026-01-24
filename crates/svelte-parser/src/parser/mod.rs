pub mod attribute;
pub mod block;
pub mod bracket;
pub mod comment;
pub mod css;
pub mod element;
pub mod expression;
pub mod fragment;
pub mod html_entities;
pub mod script;
pub mod swc_parse;
pub mod tag;
pub mod text;

use winnow::stream::{LocatingSlice, Stateful};

use crate::context::ParseContext;

pub type InputSource<'i> = LocatingSlice<&'i str>;
pub type ParserInput<'i> = Stateful<InputSource<'i>, ParseContext>;
