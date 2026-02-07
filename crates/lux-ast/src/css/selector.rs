use crate::common::Span;

#[derive(Debug)]
pub struct SelectorList<'a> {
    pub span: Span,
    pub children: Vec<ComplexSelector<'a>>,
}

#[derive(Debug)]
pub struct ComplexSelector<'a> {
    pub span: Span,
    pub children: Vec<RelativeSelector<'a>>,
    // metadata (analysis phase)
    pub is_global: bool,
    pub used: bool,
}

#[derive(Debug)]
pub struct RelativeSelector<'a> {
    pub span: Span,
    pub combinator: Option<Combinator>,
    pub selectors: Vec<SimpleSelector<'a>>,
    // metadata (analysis phase)
    pub is_global: bool,
    pub is_global_like: bool,
    pub scoped: bool,
}

#[derive(Debug)]
pub enum SimpleSelector<'a> {
    TypeSelector(TypeSelector<'a>),
    IdSelector(IdSelector<'a>),
    ClassSelector(ClassSelector<'a>),
    AttributeSelector(AttributeSelector<'a>),
    PseudoElementSelector(PseudoElementSelector<'a>),
    PseudoClassSelector(PseudoClassSelector<'a>),
    Percentage(Percentage<'a>),
    Nth(Nth<'a>),
    NestingSelector(NestingSelector),
}

#[derive(Debug)]
pub struct TypeSelector<'a> {
    pub span: Span,
    pub name: &'a str,
}

#[derive(Debug)]
pub struct IdSelector<'a> {
    pub span: Span,
    pub name: &'a str,
}

#[derive(Debug)]
pub struct ClassSelector<'a> {
    pub span: Span,
    pub name: &'a str,
}

#[derive(Debug)]
pub struct PseudoElementSelector<'a> {
    pub span: Span,
    pub name: &'a str,
}

#[derive(Debug)]
pub struct Percentage<'a> {
    pub span: Span,
    pub value: &'a str,
}

#[derive(Debug)]
pub struct Nth<'a> {
    pub span: Span,
    pub value: &'a str,
}

#[derive(Debug)]
pub struct NestingSelector {
    pub span: Span,
}

#[derive(Debug)]
pub struct AttributeSelector<'a> {
    pub span: Span,
    pub name: &'a str,
    pub matcher: Option<&'a str>,
    pub value: Option<&'a str>,
    pub flags: Option<&'a str>,
}

#[derive(Debug)]
pub struct PseudoClassSelector<'a> {
    pub span: Span,
    pub name: &'a str,
    pub args: Option<SelectorList<'a>>,
}

#[derive(Debug)]
pub struct Combinator {
    pub span: Span,
    pub kind: CombinatorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombinatorKind {
    Descendant,
    Child,
    NextSibling,
    SubsequentSibling,
    Column,
}
