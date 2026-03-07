use lux_ast::analysis::{
    AnalysisNodeKind, AnalysisTables, ComplexSelectorAnalysis, RelativeSelectorAnalysis, SpanKey,
};
use lux_ast::css::selector::{
    Combinator, CombinatorKind, ComplexSelector, RelativeSelector, SelectorList, SimpleSelector,
};
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::root::{Fragment, FragmentNode};
use lux_ast::template::tag::TextOrExpressionTag;

pub(super) fn render_selector_list(
    selector_list: &SelectorList<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    fragment: &Fragment<'_>,
) -> String {
    render_selector_list_with_scope(selector_list, analysis, scope_class, fragment, true)
}

fn render_selector_list_with_scope(
    selector_list: &SelectorList<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    fragment: &Fragment<'_>,
    scope_enabled: bool,
) -> String {
    let rendered = selector_list
        .children
        .iter()
        .map(|complex| {
            complex_is_used_or_maybe_used(complex, analysis, fragment).then(|| {
                render_complex_selector(complex, analysis, scope_class, fragment, scope_enabled)
            })
        })
        .collect::<Vec<_>>();

    let mut output = String::new();
    for (index, selector) in rendered.into_iter().enumerate() {
        let Some(selector) = selector.filter(|selector| !selector.is_empty()) else {
            continue;
        };

        if output.is_empty() {
            if index > 0 {
                output.push(' ');
            }
        } else {
            output.push_str(", ");
        }

        output.push_str(&selector);
    }

    output
}

fn render_complex_selector(
    complex: &ComplexSelector<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    fragment: &Fragment<'_>,
    scope_enabled: bool,
) -> String {
    let mut output = String::new();
    let mut has_relative = false;
    let mut specificity_bumped = false;

    for relative in &complex.children {
        let relative_output = render_relative_selector(
            relative,
            analysis,
            scope_class,
            fragment,
            scope_enabled,
            &mut specificity_bumped,
        );
        if relative_output.is_empty() {
            continue;
        }

        if has_relative {
            output.push_str(&render_combinator(relative.combinator.as_ref()));
        } else if let Some(combinator) = &relative.combinator {
            output.push_str(&render_combinator(Some(combinator)));
        }

        output.push_str(&relative_output);
        has_relative = true;
    }

    output
}

fn render_relative_selector(
    relative: &RelativeSelector<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    fragment: &Fragment<'_>,
    scope_enabled: bool,
    specificity_bumped: &mut bool,
) -> String {
    let relative_analysis = relative_analysis(relative, analysis);

    let mut rendered = relative
        .selectors
        .iter()
        .map(|selector| {
            render_simple_selector(selector, analysis, scope_class, fragment, scope_enabled)
        })
        .collect::<Vec<_>>();

    if scope_enabled && relative_analysis.scoped && can_apply_scope(relative) {
        let scope_modifier = if *specificity_bumped {
            format!(":where(.{scope_class})")
        } else {
            format!(".{scope_class}")
        };
        *specificity_bumped = true;

        apply_scope_modifier(&mut rendered, &scope_modifier);
    }

    rendered
        .into_iter()
        .map(|selector| selector.text)
        .collect::<String>()
}

fn can_apply_scope(relative: &RelativeSelector<'_>) -> bool {
    if relative
        .selectors
        .iter()
        .any(|selector| matches!(selector, SimpleSelector::NestingSelector(_)))
    {
        return false;
    }

    if relative.selectors.len() == 1 {
        return !matches!(
            &relative.selectors[0],
            SimpleSelector::PseudoClassSelector(pseudo_class)
                if pseudo_class.name == "is" || pseudo_class.name == "where"
        );
    }

    true
}

fn apply_scope_modifier(rendered: &mut [RenderedSimpleSelector], modifier: &str) {
    if rendered.is_empty() {
        return;
    }

    let mut prepended = false;

    for index in (0..rendered.len()).rev() {
        match rendered[index].kind {
            RenderedSelectorKind::PseudoElement => continue,
            RenderedSelectorKind::PseudoRootOrHost => continue,
            RenderedSelectorKind::PseudoOther => {
                if index == 0 {
                    rendered[index].text = format!("{modifier}{}", rendered[index].text);
                    prepended = true;
                    break;
                }
                continue;
            }
            RenderedSelectorKind::TypeUniversal => {
                rendered[index].text = modifier.to_string();
                prepended = true;
                break;
            }
            _ => {
                rendered[index].text.push_str(modifier);
                prepended = true;
                break;
            }
        }
    }

    if !prepended {
        rendered[0].text = format!("{modifier}{}", rendered[0].text);
    }
}

fn render_simple_selector(
    selector: &SimpleSelector<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    fragment: &Fragment<'_>,
    scope_enabled: bool,
) -> RenderedSimpleSelector {
    match selector {
        SimpleSelector::TypeSelector(type_selector) => RenderedSimpleSelector {
            kind: if type_selector.name == "*" {
                RenderedSelectorKind::TypeUniversal
            } else {
                RenderedSelectorKind::Other
            },
            text: type_selector.name.to_string(),
        },
        SimpleSelector::IdSelector(id_selector) => RenderedSimpleSelector {
            kind: RenderedSelectorKind::Other,
            text: format!("#{}", id_selector.name),
        },
        SimpleSelector::ClassSelector(class_selector) => RenderedSimpleSelector {
            kind: RenderedSelectorKind::Other,
            text: format!(".{}", class_selector.name),
        },
        SimpleSelector::AttributeSelector(attribute_selector) => {
            let mut text = String::from("[");
            text.push_str(attribute_selector.name);

            if let Some(matcher) = attribute_selector.matcher {
                text.push_str(matcher);
            }

            if let Some(value) = attribute_selector.value {
                text.push_str(value);
            }

            if let Some(flags) = attribute_selector.flags {
                text.push(' ');
                text.push_str(flags);
            }

            text.push(']');

            RenderedSimpleSelector {
                kind: RenderedSelectorKind::Other,
                text,
            }
        }
        SimpleSelector::PseudoElementSelector(pseudo_element) => RenderedSimpleSelector {
            kind: RenderedSelectorKind::PseudoElement,
            text: format!("::{}", pseudo_element.name),
        },
        SimpleSelector::PseudoClassSelector(pseudo_class) => {
            if pseudo_class.name == "global" {
                if let Some(args) = &pseudo_class.args {
                    return RenderedSimpleSelector {
                        kind: RenderedSelectorKind::Other,
                        text: render_selector_list_with_scope(
                            args,
                            analysis,
                            scope_class,
                            fragment,
                            false,
                        ),
                    };
                }

                return RenderedSimpleSelector {
                    kind: RenderedSelectorKind::PseudoOther,
                    text: String::new(),
                };
            }

            let mut text = format!(":{}", pseudo_class.name);
            if let Some(args) = &pseudo_class.args {
                text.push('(');
                text.push_str(&render_selector_list_with_scope(
                    args,
                    analysis,
                    scope_class,
                    fragment,
                    scope_enabled,
                ));
                text.push(')');
            }

            RenderedSimpleSelector {
                kind: if pseudo_class.name == "root" || pseudo_class.name == "host" {
                    RenderedSelectorKind::PseudoRootOrHost
                } else {
                    RenderedSelectorKind::PseudoOther
                },
                text,
            }
        }
        SimpleSelector::Percentage(percentage) => RenderedSimpleSelector {
            kind: RenderedSelectorKind::Other,
            text: percentage.value.to_string(),
        },
        SimpleSelector::Nth(nth) => RenderedSimpleSelector {
            kind: RenderedSelectorKind::Other,
            text: nth.value.to_string(),
        },
        SimpleSelector::NestingSelector(_) => RenderedSimpleSelector {
            kind: RenderedSelectorKind::Nesting,
            text: "&".to_string(),
        },
    }
}

fn complex_is_used_or_maybe_used(
    complex: &ComplexSelector<'_>,
    analysis: &AnalysisTables,
    fragment: &Fragment<'_>,
) -> bool {
    let complex_analysis = analysis
        .complex_selectors
        .get(&SpanKey::new(
            AnalysisNodeKind::ComplexSelector,
            complex.span,
        ))
        .cloned()
        .unwrap_or(ComplexSelectorAnalysis {
            is_global: false,
            used: false,
        });
    if complex_analysis.used || complex_analysis.is_global {
        return true;
    }
    if fragment.dynamic || complex.children.len() != 1 {
        return true;
    }

    fragment_contains_matching_element(fragment, &complex.children[0], analysis)
}

fn fragment_contains_matching_element(
    fragment: &Fragment<'_>,
    relative: &RelativeSelector<'_>,
    analysis: &AnalysisTables,
) -> bool {
    fragment
        .nodes
        .iter()
        .any(|node| node_contains_matching_element(node, relative, analysis))
}

fn node_contains_matching_element(
    node: &FragmentNode<'_>,
    relative: &RelativeSelector<'_>,
    analysis: &AnalysisTables,
) -> bool {
    match node {
        FragmentNode::RegularElement(element) => {
            relative_targets_element(relative, analysis, element.name, &element.attributes)
                || fragment_contains_matching_element(&element.fragment, relative, analysis)
        }
        FragmentNode::TitleElement(element) => {
            fragment_contains_matching_element(&element.fragment, relative, analysis)
        }
        FragmentNode::SvelteFragment(element) => {
            fragment_contains_matching_element(&element.fragment, relative, analysis)
        }
        FragmentNode::SvelteBody(element) => {
            fragment_contains_matching_element(&element.fragment, relative, analysis)
        }
        FragmentNode::SvelteWindow(element) => {
            fragment_contains_matching_element(&element.fragment, relative, analysis)
        }
        FragmentNode::SvelteDocument(element) => {
            fragment_contains_matching_element(&element.fragment, relative, analysis)
        }
        FragmentNode::SvelteBoundary(element) => {
            fragment_contains_matching_element(&element.fragment, relative, analysis)
        }
        FragmentNode::KeyBlock(block) => {
            fragment_contains_matching_element(&block.fragment, relative, analysis)
        }
        _ => false,
    }
}

fn relative_targets_element(
    relative: &RelativeSelector<'_>,
    analysis: &AnalysisTables,
    name: &str,
    attributes: &[AttributeNode<'_>],
) -> bool {
    relative
        .selectors
        .iter()
        .all(|selector| simple_selector_targets_element(selector, analysis, name, attributes))
}

fn simple_selector_targets_element(
    selector: &SimpleSelector<'_>,
    analysis: &AnalysisTables,
    name: &str,
    attributes: &[AttributeNode<'_>],
) -> bool {
    match selector {
        SimpleSelector::TypeSelector(type_selector) => {
            type_selector.name == "*" || type_selector.name.eq_ignore_ascii_case(name)
        }
        SimpleSelector::IdSelector(id_selector) => {
            static_attribute_value(attributes, "id").is_some_and(|value| value == id_selector.name)
        }
        SimpleSelector::ClassSelector(class_selector) => {
            attributes.iter().any(|attribute| match attribute {
                AttributeNode::Attribute(attribute) if attribute.name == "class" => {
                    static_class_tokens(&attribute.value)
                        .iter()
                        .any(|token| *token == class_selector.name)
                }
                AttributeNode::ClassDirective(directive) => directive.name == class_selector.name,
                _ => false,
            })
        }
        SimpleSelector::AttributeSelector(attribute_selector) => {
            let Some(value) = static_attribute_value(attributes, attribute_selector.name) else {
                return attributes.iter().any(|attribute| {
                    matches!(attribute, AttributeNode::Attribute(attribute) if attribute.name == attribute_selector.name)
                });
            };

            attribute_selector
                .value
                .is_none_or(|expected| value == strip_attribute_selector_quotes(expected))
        }
        SimpleSelector::PseudoClassSelector(pseudo_class) => match pseudo_class.name {
            "global" | "root" | "host" => false,
            "is" | "where" => pseudo_class
                .args
                .as_ref()
                .is_none_or(|args| selector_list_matches_element(args, analysis, name, attributes)),
            "not" => pseudo_class.args.as_ref().is_none_or(|args| {
                !selector_list_matches_element(args, analysis, name, attributes)
            }),
            _ => true,
        },
        SimpleSelector::PseudoElementSelector(_)
        | SimpleSelector::Percentage(_)
        | SimpleSelector::Nth(_)
        | SimpleSelector::NestingSelector(_) => true,
    }
}

fn selector_list_matches_element(
    selector_list: &SelectorList<'_>,
    analysis: &AnalysisTables,
    name: &str,
    attributes: &[AttributeNode<'_>],
) -> bool {
    selector_list.children.iter().any(|complex| {
        complex.children.len() == 1
            && relative_targets_element(&complex.children[0], analysis, name, attributes)
    })
}

fn static_attribute_value<'a>(attributes: &'a [AttributeNode<'a>], name: &str) -> Option<&'a str> {
    attributes.iter().find_map(|attribute| {
        let AttributeNode::Attribute(attribute) = attribute else {
            return None;
        };
        if attribute.name != name {
            return None;
        }
        match &attribute.value {
            AttributeValue::True => Some(""),
            AttributeValue::ExpressionTag(_) => None,
            AttributeValue::Sequence(chunks) => static_text_chunks(chunks),
        }
    })
}

fn static_class_tokens<'a>(value: &'a AttributeValue<'a>) -> Vec<&'a str> {
    match value {
        AttributeValue::True | AttributeValue::ExpressionTag(_) => Vec::new(),
        AttributeValue::Sequence(chunks) => static_text_chunks(chunks)
            .map(|text| text.split_whitespace().collect::<Vec<_>>())
            .unwrap_or_default(),
    }
}

fn static_text_chunks<'a>(chunks: &'a [TextOrExpressionTag<'a>]) -> Option<&'a str> {
    if chunks
        .iter()
        .any(|chunk| matches!(chunk, TextOrExpressionTag::ExpressionTag(_)))
    {
        return None;
    }

    if chunks.len() != 1 {
        return None;
    }

    match &chunks[0] {
        TextOrExpressionTag::Text(text) => Some(text.raw),
        TextOrExpressionTag::ExpressionTag(_) => None,
    }
}

fn strip_attribute_selector_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|rest| rest.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn render_combinator(combinator: Option<&Combinator>) -> String {
    match combinator.map(|combinator| combinator.kind) {
        Some(CombinatorKind::Descendant) => " ".to_string(),
        Some(CombinatorKind::Child) => " > ".to_string(),
        Some(CombinatorKind::NextSibling) => " + ".to_string(),
        Some(CombinatorKind::SubsequentSibling) => " ~ ".to_string(),
        Some(CombinatorKind::Column) => " || ".to_string(),
        None => " ".to_string(),
    }
}

fn relative_analysis<'a>(
    relative: &'a RelativeSelector<'_>,
    analysis: &'a AnalysisTables,
) -> RelativeSelectorAnalysis {
    analysis
        .relative_selectors
        .get(&SpanKey::new(
            AnalysisNodeKind::RelativeSelector,
            relative.span,
        ))
        .cloned()
        .unwrap_or(RelativeSelectorAnalysis {
            is_global: false,
            is_global_like: false,
            scoped: false,
        })
}

#[derive(Debug)]
struct RenderedSimpleSelector {
    kind: RenderedSelectorKind,
    text: String,
}

#[derive(Debug, Clone, Copy)]
enum RenderedSelectorKind {
    TypeUniversal,
    PseudoRootOrHost,
    PseudoOther,
    PseudoElement,
    Nesting,
    Other,
}
