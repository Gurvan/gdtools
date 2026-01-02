pub mod basic;
pub mod design;
pub mod format;
pub mod naming;
pub mod style;

use crate::lint::Rule;

pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // Naming rules
        Box::new(naming::FunctionNameRule::default()),
        Box::new(naming::ClassNameRule::default()),
        Box::new(naming::SignalNameRule::default()),
        Box::new(naming::ConstantNameRule::default()),
        Box::new(naming::EnumNameRule::default()),
        Box::new(naming::EnumElementNameRule::default()),
        Box::new(naming::FunctionArgumentNameRule::default()),
        Box::new(naming::LoopVariableNameRule::default()),
        Box::new(naming::SubClassNameRule::default()),
        Box::new(naming::LoadConstantNameRule::default()),
        Box::new(naming::ClassVariableNameRule::default()),
        Box::new(naming::ClassLoadVariableNameRule::default()),
        Box::new(naming::FunctionVariableNameRule::default()),
        Box::new(naming::FunctionPreloadVariableNameRule::default()),
        // Format rules
        Box::new(format::MaxLineLengthRule::default()),
        Box::new(format::TrailingWhitespaceRule::default()),
        Box::new(format::MixedTabsSpacesRule::default()),
        Box::new(format::MaxFileLinesRule::default()),
        // Basic rules
        Box::new(basic::UnnecessaryPassRule::default()),
        Box::new(basic::UnusedArgumentRule::default()),
        Box::new(basic::ComparisonWithItselfRule::default()),
        Box::new(basic::DuplicatedLoadRule::default()),
        Box::new(basic::ExpressionNotAssignedRule::default()),
        // Design rules
        Box::new(design::MaxFunctionArgsRule::default()),
        Box::new(design::MaxReturnsRule::default()),
        Box::new(design::MaxPublicMethodsRule::default()),
        // Style rules
        Box::new(style::ClassDefinitionsOrderRule::default()),
        Box::new(style::NoElifReturnRule::default()),
        Box::new(style::NoElseReturnRule::default()),
    ]
}
