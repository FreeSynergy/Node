// Steps — UI-agnostic installation wizard steps.
//
// Each step captures one phase of the setup process.
// CLI, TUI, or GUI can drive these without coupling to a specific UI framework.

pub mod iam;
pub mod languages;
pub mod network;
pub mod proxy;
pub mod services;
pub mod store;
pub mod timezone;

/// A single wizard step. UI-agnostic: CLI, TUI, or GUI can drive these.
///
/// Each step has an input type, an output type, a display title, and a
/// validation function that returns a list of human-readable error strings.
pub trait WizardStep {
    /// The data collected by this step.
    type Input;
    /// The data produced after completing this step.
    type Output;

    /// Short display title shown in the wizard header.
    fn title(&self) -> &str;

    /// Validate the given input and return any validation errors.
    ///
    /// Returns an empty vec when the input is valid.
    fn validate(&self, input: &Self::Input) -> Vec<String>;
}
