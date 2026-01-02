pub mod basic;
pub mod format;
pub mod naming;
pub mod style;

use crate::lint::Rule;

pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(naming::FunctionNameRule::default()),
        Box::new(naming::ClassNameRule::default()),
        Box::new(naming::SignalNameRule::default()),
        Box::new(naming::ConstantNameRule::default()),
        Box::new(naming::VariableNameRule::default()),
        Box::new(naming::EnumNameRule::default()),
        Box::new(naming::EnumElementNameRule::default()),
        Box::new(format::MaxLineLengthRule::default()),
        Box::new(format::TrailingWhitespaceRule::default()),
        Box::new(format::MixedTabsSpacesRule::default()),
        Box::new(basic::UnnecessaryPassRule::default()),
        Box::new(basic::UnusedArgumentRule::default()),
        Box::new(basic::ComparisonWithItselfRule::default()),
        Box::new(style::ClassDefinitionsOrderRule::default()),
    ]
}
