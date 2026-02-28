use lux_ast::analysis::{AnalysisNodeKind, AnalysisTables, RelativeSelectorAnalysis, SpanKey};
use lux_ast::css::selector::{
    Combinator, CombinatorKind, ComplexSelector, RelativeSelector, SelectorList, SimpleSelector,
};

pub(super) fn render_selector_list(
    selector_list: &SelectorList<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
) -> String {
    render_selector_list_with_scope(selector_list, analysis, scope_class, true)
}

fn render_selector_list_with_scope(
    selector_list: &SelectorList<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    scope_enabled: bool,
) -> String {
    selector_list
        .children
        .iter()
        .map(|complex| render_complex_selector(complex, analysis, scope_class, scope_enabled))
        .filter(|selector| !selector.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_complex_selector(
    complex: &ComplexSelector<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
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
    scope_enabled: bool,
    specificity_bumped: &mut bool,
) -> String {
    let relative_analysis = relative_analysis(relative, analysis);

    let mut rendered = relative
        .selectors
        .iter()
        .map(|selector| render_simple_selector(selector, analysis, scope_class, scope_enabled))
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
                        text: render_selector_list_with_scope(args, analysis, scope_class, false),
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
