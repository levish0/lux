#[test]
fn test_oxc_grammar_boundary() {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser as OxcParser;
    use oxc_span::{GetSpan, SourceType};

    let allocator = Allocator::default();

    // OXC stops at expression boundary - grammar-based, not byte scanning
    let r = OxcParser::new(&allocator, "x + 1} rest", SourceType::mjs()).parse_expression();
    assert!(r.is_ok());
    assert_eq!(r.expect("valid expression").span().end, 5);

    let r = OxcParser::new(&allocator, "foo(1, 2)} more", SourceType::mjs()).parse_expression();
    assert!(r.is_ok());
    assert_eq!(r.expect("valid expression").span().end, 9);

    let r = OxcParser::new(&allocator, "{ a: 1 }} rest", SourceType::mjs()).parse_expression();
    assert!(r.is_ok());
    assert_eq!(r.expect("valid expression").span().end, 8);

    // Regex with } in char class - grammar handles correctly
    let r = OxcParser::new(&allocator, "/[}]/.test(x)} rest", SourceType::mjs()).parse_expression();
    assert!(r.is_ok());
    assert_eq!(r.expect("valid expression").span().end, 13);

    // TS as expression consumed by OXC
    let r = OxcParser::new(&allocator, "items as item}", SourceType::ts()).parse_expression();
    assert!(r.is_ok());
    assert_eq!(r.expect("valid TS as expression").span().end, 13);

    // JS mode with `as` - OXC fails (needs fallback)
    let r = OxcParser::new(&allocator, "items as item}", SourceType::mjs()).parse_expression();
    assert!(r.is_err());
}
