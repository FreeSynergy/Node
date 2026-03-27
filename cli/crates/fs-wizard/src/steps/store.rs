// Store configuration step — set the store URL and active namespaces.

use super::WizardStep;

/// Input data for the store configuration step.
#[derive(Debug, Clone)]
pub struct StoreInput {
    /// Base URL of the `FreeSynergy` store (e.g. "<https://raw.githubusercontent.com/FreeSynergy/Store/main>").
    pub url: String,
    /// Store namespaces to activate (e.g. `["Node", "Community"]`).
    pub namespaces: Vec<String>,
}

impl Default for StoreInput {
    fn default() -> Self {
        Self {
            url: "https://raw.githubusercontent.com/FreeSynergy/Store/main".to_string(),
            namespaces: vec!["Node".to_string()],
        }
    }
}

/// Wizard step that configures the module store connection.
pub struct StoreStep;

impl StoreStep {
    /// Create a new `StoreStep`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for StoreStep {
    fn default() -> Self {
        Self::new()
    }
}

impl WizardStep for StoreStep {
    type Input = StoreInput;
    type Output = StoreInput;

    fn title(&self) -> &'static str {
        "Store Configuration"
    }

    fn validate(&self, input: &Self::Input) -> Vec<String> {
        let mut errors = Vec::new();

        if input.url.trim().is_empty() {
            errors.push("Store URL is required.".to_string());
        } else if !input.url.starts_with("https://") && !input.url.starts_with("http://") {
            errors.push("Store URL must start with http:// or https://".to_string());
        }

        if input.namespaces.is_empty() {
            errors.push("At least one namespace is required.".to_string());
        }

        for ns in &input.namespaces {
            if ns.trim().is_empty() {
                errors.push("Namespace names must not be empty.".to_string());
                break;
            }
        }

        errors
    }
}
