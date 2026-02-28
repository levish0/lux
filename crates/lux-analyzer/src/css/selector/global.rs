use lux_ast::css::selector::{RelativeSelector, SimpleSelector};

pub(super) fn starts_with_global_block(relative_selector: &RelativeSelector<'_>) -> bool {
    relative_selector
        .selectors
        .first()
        .is_some_and(is_global_block_selector)
}

pub(super) fn is_relative_global(relative_selector: &RelativeSelector<'_>) -> bool {
    let Some(SimpleSelector::PseudoClassSelector(first)) = relative_selector.selectors.first()
    else {
        return false;
    };

    if first.name != "global" {
        return false;
    }

    first.args.is_none()
        || relative_selector.selectors.iter().all(|selector| {
            is_unscoped_pseudo_class(selector)
                || matches!(selector, SimpleSelector::PseudoElementSelector(_))
        })
}

pub(super) fn is_unscoped_pseudo_class(selector: &SimpleSelector<'_>) -> bool {
    let SimpleSelector::PseudoClassSelector(pseudo_class) = selector else {
        return false;
    };

    let cannot_be_scoped = pseudo_class.name != "has"
        && pseudo_class.name != "is"
        && pseudo_class.name != "where"
        && (pseudo_class.name != "not"
            || pseudo_class.args.as_ref().is_none_or(|args| {
                args.children
                    .iter()
                    .all(|complex| complex.children.len() == 1)
            }));

    cannot_be_scoped
        || pseudo_class.args.as_ref().is_none_or(|args| {
            args.children
                .iter()
                .all(|complex| complex.children.iter().all(is_relative_global))
        })
}

fn is_global_block_selector(selector: &SimpleSelector<'_>) -> bool {
    matches!(
        selector,
        SimpleSelector::PseudoClassSelector(pseudo_class)
            if pseudo_class.name == "global" && pseudo_class.args.is_none()
    )
}
