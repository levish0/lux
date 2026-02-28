use lux_ast::template::root::FragmentNode;
use lux_parser::parse;
use oxc_allocator::Allocator;

#[test]
fn top_level_script_is_extracted_to_root_instance() {
    let allocator = Allocator::default();
    let result = parse("<script>let answer = 42;</script>", &allocator, false);

    assert!(result.root.instance.is_some());
    assert!(result.root.fragment.nodes.is_empty());
}

#[test]
fn script_inside_if_block_stays_as_regular_element() {
    let allocator = Allocator::default();
    let result = parse(
        "{#if ok}<script>console.log(1)</script>{/if}",
        &allocator,
        false,
    );

    assert!(result.root.instance.is_none());
    assert!(result.root.css.is_none());

    let node = result
        .root
        .fragment
        .nodes
        .first()
        .expect("missing root node");
    let if_block = match node {
        FragmentNode::IfBlock(block) => block,
        _ => panic!("expected IfBlock"),
    };

    let inner_node = if_block
        .consequent
        .nodes
        .first()
        .expect("missing node inside if block");
    match inner_node {
        FragmentNode::RegularElement(element) => assert_eq!(element.name, "script"),
        _ => panic!("expected nested RegularElement"),
    }
}

#[test]
fn script_inside_component_stays_nested() {
    let allocator = Allocator::default();
    let result = parse(
        "<Widget><script>console.log(1)</script></Widget>",
        &allocator,
        false,
    );

    assert!(result.root.instance.is_none());

    let node = result
        .root
        .fragment
        .nodes
        .first()
        .expect("missing root node");
    let component = match node {
        FragmentNode::Component(component) => component,
        _ => panic!("expected Component"),
    };

    let inner_node = component
        .fragment
        .nodes
        .first()
        .expect("missing node inside component");
    match inner_node {
        FragmentNode::RegularElement(element) => assert_eq!(element.name, "script"),
        _ => panic!("expected nested RegularElement"),
    }
}

#[test]
fn script_lang_ts_enables_typescript_mode_automatically() {
    let allocator = Allocator::default();
    let result = parse(
        "<script lang=\"ts\">let answer: number = 42;</script>{answer as number}",
        &allocator,
        false,
    );

    assert!(result.root.ts);
}

#[test]
fn commented_script_lang_ts_does_not_enable_typescript_mode() {
    let allocator = Allocator::default();
    let result = parse(
        "<!-- <script lang=\"ts\"></script> --><script>let answer = 42;</script>",
        &allocator,
        false,
    );

    assert!(!result.root.ts);
}

#[test]
fn explicit_ts_flag_still_forces_typescript_mode() {
    let allocator = Allocator::default();
    let result = parse("<script>let answer = 42;</script>", &allocator, true);

    assert!(result.root.ts);
}
