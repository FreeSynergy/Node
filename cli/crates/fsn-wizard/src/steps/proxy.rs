// Proxy setup step — configure Zentinel (Traefik-based reverse proxy).

use super::WizardStep;

/// Input data for the proxy configuration step.
#[derive(Debug, Clone)]
pub struct ProxyInput {
    /// Primary domain served by the proxy (e.g. "example.com").
    pub domain: String,
    /// Email address used for ACME / Let's Encrypt certificate requests.
    pub acme_email: String,
    /// Whether to enable TLS (HTTPS) via ACME.
    pub use_tls: bool,
}

impl Default for ProxyInput {
    fn default() -> Self {
        Self {
            domain: String::new(),
            acme_email: String::new(),
            use_tls: true,
        }
    }
}

/// Wizard step that configures the Zentinel reverse proxy.
pub struct ProxyStep;

impl ProxyStep {
    /// Create a new `ProxyStep`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProxyStep {
    fn default() -> Self {
        Self::new()
    }
}

impl WizardStep for ProxyStep {
    type Input = ProxyInput;
    type Output = ProxyInput;

    fn title(&self) -> &str {
        "Proxy Setup (Zentinel)"
    }

    fn validate(&self, input: &Self::Input) -> Vec<String> {
        let mut errors = Vec::new();

        if input.domain.trim().is_empty() {
            errors.push("Domain is required.".to_string());
        } else if !input.domain.contains('.') {
            errors.push("Domain must contain at least one dot (e.g. example.com).".to_string());
        }

        if input.use_tls {
            if input.acme_email.trim().is_empty() {
                errors.push("ACME email is required when TLS is enabled.".to_string());
            } else if !input.acme_email.contains('@') {
                errors.push("ACME email must be a valid email address.".to_string());
            }
        }

        errors
    }
}
