use oxc_ast::ast::{
    AssignmentExpression, AssignmentTargetPropertyIdentifier, AssignmentTargetPropertyProperty,
    AssignmentTargetRest, AssignmentTargetWithDefault, Expression, IdentifierReference,
    SimpleAssignmentTarget, UpdateExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::operator::AssignmentOperator;

use super::context::TemplateAnalyzerContext;

#[derive(Debug, Clone, Copy)]
struct AccessMode {
    is_read: bool,
    is_write: bool,
}

const READ: AccessMode = AccessMode {
    is_read: true,
    is_write: false,
};
const WRITE: AccessMode = AccessMode {
    is_read: false,
    is_write: true,
};
const READ_WRITE: AccessMode = AccessMode {
    is_read: true,
    is_write: true,
};

pub(super) fn analyze_expression(
    expression: &Expression<'_>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    let mut collector = ExpressionReferenceCollector::new(context);
    collector.visit_expression(expression);
}

struct ExpressionReferenceCollector<'ctx, 'tables> {
    context: &'ctx mut TemplateAnalyzerContext<'tables>,
    mode_stack: Vec<AccessMode>,
}

impl<'ctx, 'tables> ExpressionReferenceCollector<'ctx, 'tables> {
    fn new(context: &'ctx mut TemplateAnalyzerContext<'tables>) -> Self {
        Self {
            context,
            mode_stack: vec![READ],
        }
    }

    fn current_mode(&self) -> AccessMode {
        *self
            .mode_stack
            .last()
            .expect("expression collector mode stack should never be empty")
    }

    fn with_mode(&mut self, mode: AccessMode, f: impl FnOnce(&mut Self)) {
        self.mode_stack.push(mode);
        f(self);
        self.mode_stack
            .pop()
            .expect("expression collector mode stack should never be empty");
    }

    fn record_identifier_reference(&mut self, identifier: &IdentifierReference<'_>) {
        let mode = self.current_mode();
        self.context.add_reference(
            identifier.name.as_str(),
            identifier.span,
            mode.is_read,
            mode.is_write,
        );
    }
}

impl<'a> Visit<'a> for ExpressionReferenceCollector<'_, '_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        self.record_identifier_reference(it);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        let left_mode = if it.operator == AssignmentOperator::Assign {
            WRITE
        } else {
            READ_WRITE
        };

        self.with_mode(left_mode, |collector| {
            collector.visit_assignment_target(&it.left);
        });

        self.with_mode(READ, |collector| {
            collector.visit_expression(&it.right);
        });
    }

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        self.with_mode(READ_WRITE, |collector| {
            collector.visit_simple_assignment_target(&it.argument);
        });
    }

    fn visit_simple_assignment_target(&mut self, it: &SimpleAssignmentTarget<'a>) {
        match it {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                self.record_identifier_reference(identifier);
            }
            SimpleAssignmentTarget::ComputedMemberExpression(member) => {
                self.with_mode(READ, |collector| {
                    collector.visit_expression(&member.object);
                    collector.visit_expression(&member.expression);
                });
            }
            SimpleAssignmentTarget::StaticMemberExpression(member) => {
                self.with_mode(READ, |collector| {
                    collector.visit_expression(&member.object);
                });
            }
            SimpleAssignmentTarget::PrivateFieldExpression(member) => {
                self.with_mode(READ, |collector| {
                    collector.visit_expression(&member.object);
                });
            }
            _ => {
                if let Some(expression) = it.get_expression() {
                    if let Some(target) = expression.as_simple_assignment_target() {
                        self.visit_simple_assignment_target(target);
                    } else {
                        self.visit_expression(expression);
                    }
                }
            }
        }
    }

    fn visit_assignment_target_with_default(&mut self, it: &AssignmentTargetWithDefault<'a>) {
        self.with_mode(WRITE, |collector| {
            collector.visit_assignment_target(&it.binding);
        });
        self.with_mode(READ, |collector| {
            collector.visit_expression(&it.init);
        });
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        it: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.with_mode(WRITE, |collector| {
            collector.visit_identifier_reference(&it.binding);
        });

        if let Some(init) = &it.init {
            self.with_mode(READ, |collector| {
                collector.visit_expression(init);
            });
        }
    }

    fn visit_assignment_target_property_property(
        &mut self,
        it: &AssignmentTargetPropertyProperty<'a>,
    ) {
        self.with_mode(READ, |collector| {
            collector.visit_property_key(&it.name);
        });

        self.with_mode(WRITE, |collector| {
            collector.visit_assignment_target_maybe_default(&it.binding);
        });
    }

    fn visit_assignment_target_rest(&mut self, it: &AssignmentTargetRest<'a>) {
        self.with_mode(WRITE, |collector| {
            collector.visit_assignment_target(&it.target);
        });
    }

    fn visit_expression(&mut self, it: &Expression<'a>) {
        walk::walk_expression(self, it);
    }
}
