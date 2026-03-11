// Language selection.
//
// Pattern: Value Object — Lang carries the active UI language and knows how to
// produce its display label. Language cycling is handled by AppState::cycle_lang()
// because it needs access to the list of loaded languages.
//
// Lang is Copy: En is a zero-size variant; Dynamic holds a raw pointer to a
// leaked DynamicLang allocation, which is also Copy (&'static T is Copy).

use crate::i18n::DynamicLang;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Dynamic(&'static DynamicLang),
}

impl Lang {
    pub fn label(self) -> &'static str {
        match self {
            Lang::En         => "EN",
            Lang::Dynamic(d) => d.code_upper,
        }
    }

    /// Language code, e.g. "en", "de", "fr".
    pub fn code(self) -> &'static str {
        match self {
            Lang::En         => "en",
            Lang::Dynamic(d) => d.code,
        }
    }
}
