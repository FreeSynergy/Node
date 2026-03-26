// fs-auth/src/oauth2.rs — OAuth2 Authorization Code Flow (feature: oauth2).

use fs_error::FsError;
use serde::{Deserialize, Serialize};

// ── OAuth2Config ──────────────────────────────────────────────────────────────

/// Configuration for an OAuth2 Authorization Code client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    /// OAuth2 `client_id`.
    pub client_id: String,
    /// OAuth2 `client_secret`.
    #[serde(skip_serializing)]
    pub client_secret: String,
    /// Authorization endpoint URL.
    pub auth_url: String,
    /// Token endpoint URL.
    pub token_url: String,
    /// Redirect URI registered with the provider.
    pub redirect_uri: String,
    /// Requested scopes.
    pub scopes: Vec<String>,
}

// ── TokenResponse ─────────────────────────────────────────────────────────────

/// Access token response from the token endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    /// The access token.
    pub access_token: String,
    /// Refresh token, if issued.
    pub refresh_token: Option<String>,
    /// Token lifetime in seconds.
    pub expires_in: Option<u64>,
    /// Token type (typically `"Bearer"`).
    pub token_type: String,
    /// ID token (if OIDC scope was requested).
    pub id_token: Option<String>,
}

// ── OAuth2Client ──────────────────────────────────────────────────────────────

/// OAuth2 Authorization Code flow client.
///
/// Requires feature `oauth2`.
///
/// # Example
///
/// ```rust,ignore
/// use fs_auth::oauth2::{OAuth2Client, OAuth2Config};
///
/// let config = OAuth2Config {
///     client_id: "my-app".into(),
///     client_secret: "secret".into(),
///     auth_url: "https://accounts.example.com/oauth/authorize".into(),
///     token_url: "https://accounts.example.com/oauth/token".into(),
///     redirect_uri: "https://myapp.example.com/callback".into(),
///     scopes: vec!["openid".into(), "profile".into()],
/// };
///
/// let client = OAuth2Client::new(config);
/// let url = client.authorization_url("random-state-token");
/// // redirect user to url …
///
/// // On callback:
/// let tokens = client.exchange_code("auth-code", "random-state-token").await?;
/// ```
pub struct OAuth2Client {
    config: OAuth2Config,
    http: reqwest::Client,
}

impl OAuth2Client {
    /// Create a new client.
    pub fn new(config: OAuth2Config) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    /// Build the authorization URL to redirect the user to.
    ///
    /// `state` should be a random, unguessable string stored in the session
    /// to prevent CSRF attacks.
    pub fn authorization_url(&self, state: &str) -> String {
        let scopes = self.config.scopes.join(" ");
        let url = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
            self.config.auth_url,
            urlencoded(&self.config.client_id),
            urlencoded(&self.config.redirect_uri),
            urlencoded(&scopes),
            urlencoded(state),
        );
        url
    }

    /// Exchange an authorization code for access and refresh tokens.
    ///
    /// `state` must match what was passed to [`authorization_url`](OAuth2Client::authorization_url).
    pub async fn exchange_code(&self, code: &str, _state: &str) -> Result<TokenResponse, FsError> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let resp = self
            .http
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| FsError::network(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_else(|_| String::new());
            return Err(FsError::auth(format!("token exchange failed: {body}")));
        }

        resp.json::<TokenResponse>()
            .await
            .map_err(|e| FsError::parse(e.to_string()))
    }

    /// Use a refresh token to obtain a new access token.
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse, FsError> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let resp = self
            .http
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| FsError::network(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_else(|_| String::new());
            return Err(FsError::auth(format!("token refresh failed: {body}")));
        }

        resp.json::<TokenResponse>()
            .await
            .map_err(|e| FsError::parse(e.to_string()))
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn urlencoded(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                vec![c]
            }
            c => format!("%{:02X}", c as u8).chars().collect(),
        })
        .collect()
}
