use chrono::{DateTime, Utc};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, HttpClientError, HttpRequest,
    HttpResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, TokenResponse,
    TokenUrl,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::Error;

/// An access/refresh token pair obtained via the OAuth Authorization Code + PKCE flow.
/// `Serialize`/`Deserialize` so a caller can persist it between runs — the library never does
/// this itself. `refresh_token` and `expires_at` are `Option` because a refresh response isn't
/// guaranteed to include a new refresh token (reuse the old one if so) or an expiry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

fn tokens_from_response<TR: TokenResponse>(response: &TR) -> OAuthTokens {
    OAuthTokens {
        access_token: response.access_token().secret().clone(),
        refresh_token: response.refresh_token().map(|rt| rt.secret().clone()),
        expires_at: response
            .expires_in()
            .and_then(|d| chrono::Duration::from_std(d).ok())
            .map(|d| Utc::now() + d),
    }
}

/// OAuth application credentials and endpoints. Build once, then call `authorization_url` to
/// start the Authorization Code + PKCE flow.
///
/// Only the Authorization Code + PKCE grant is supported. YNAB's other grant, Implicit, returns
/// the token in a URL fragment, which a Rust backend has no way to receive.
///
/// # Examples
///
/// ```no_run
/// use rust_ynab::{Client, OAuthConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = OAuthConfig::new(
///     std::env::var("YNAB_OAUTH_CLIENT_ID")?,
///     std::env::var("YNAB_OAUTH_CLIENT_SECRET")?,
///     "https://app.ynab.com/oauth/authorize",
///     "https://app.ynab.com/oauth/token",
///     "https://your-app.example.com/callback",
/// )?;
///
/// // Redirect the user to `auth_url` and hold onto `verifier` and `csrf_token` until the
/// // callback arrives.
/// let (auth_url, verifier, csrf_token) = config.authorization_url();
/// println!("visit: {auth_url}");
///
/// // ... once your redirect handler receives the full callback URL — `verify_and_extract_code`
/// // checks `state` against `csrf_token` for you and only returns `code` if it matches:
/// # let redirect_url = "https://your-app.example.com/callback?code=code-from-the-redirect&state=...";
/// let code = OAuthConfig::verify_and_extract_code(redirect_url, &csrf_token)?;
/// let tokens = config.exchange_code(code, verifier).await?;
/// let client = Client::new(tokens.access_token)?;
///
/// // `tokens.refresh_token`/`tokens.expires_at` are both `Option` — not every response includes
/// // them. `OAuthTokens` implements `Serialize`/`Deserialize`; persisting it between runs is up
/// // to the caller. When the access token expires, refresh explicitly — there's no automatic
/// // in-`Client` refresh:
/// # let stored_refresh_token = tokens.refresh_token.unwrap();
/// let tokens = config.refresh_token(stored_refresh_token).await?;
/// let client = Client::new(tokens.access_token)?;
/// # let _ = client;
/// # Ok(())
/// # }
/// ```
pub struct OAuthConfig {
    client_id: ClientId,
    client_secret: ClientSecret,
    auth_url: AuthUrl,
    token_url: TokenUrl,
    redirect_uri: RedirectUrl,
    http_client: reqwest::Client,
}

impl OAuthConfig {
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        auth_url: impl Into<String>,
        token_url: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Result<Self, Error> {
        Ok(Self {
            client_id: ClientId::new(client_id.into()),
            client_secret: ClientSecret::new(client_secret.into()),
            auth_url: AuthUrl::new(auth_url.into())?,
            token_url: TokenUrl::new(token_url.into())?,
            redirect_uri: RedirectUrl::new(redirect_uri.into())?,
            http_client: reqwest::Client::new(),
        })
    }

    /// Adapts our own `reqwest = "0.13"` client to oauth2's HTTP-client-agnostic interface,
    /// reusing the shared `http_client` rather than constructing a fresh `reqwest::Client` per
    /// call (which would mean a new connection pool and TLS setup on every OAuth request).
    ///
    /// oauth2's bundled `reqwest` feature requires `reqwest ^0.12`, which isn't semver-compatible
    /// with this crate's reqwest version — enabling it would pull in a second copy of reqwest
    /// (and hyper, and a TLS stack) just for the OAuth token endpoint. Since this takes `&self`
    /// rather than just an `HttpRequest`, it isn't itself `Fn`-shaped — call sites wrap it in a
    /// closure (`|req| self.http_response(req)`), which satisfies oauth2's blanket
    /// `AsyncHttpClient` impl for any `Fn(HttpRequest) -> impl Future<..>`. We can't implement
    /// `AsyncHttpClient` on `reqwest::Client` directly ourselves either way — orphan rule, since
    /// neither the trait nor the type is local to this crate. Mirrors oauth2's own built-in
    /// reqwest integration, just targeting our reqwest version instead of theirs.
    async fn http_response(
        &self,
        request: HttpRequest,
    ) -> Result<HttpResponse, HttpClientError<reqwest::Error>> {
        let response = self
            .http_client
            .execute(request.try_into().map_err(Box::new)?)
            .await
            .map_err(Box::new)?;

        let mut builder = oauth2::http::Response::builder().status(response.status());
        for (name, value) in response.headers().iter() {
            builder = builder.header(name, value);
        }
        builder
            .body(response.bytes().await.map_err(Box::new)?.to_vec())
            .map_err(HttpClientError::Http)
    }

    /// Builds the URL to redirect the user to. Returns the URL alongside the PKCE verifier and
    /// CSRF token you must hold onto until the code-exchange step.
    pub fn authorization_url(&self) -> (Url, PkceCodeVerifier, CsrfToken) {
        let client = BasicClient::new(self.client_id.clone())
            .set_client_secret(self.client_secret.clone())
            .set_auth_uri(self.auth_url.clone())
            .set_token_uri(self.token_url.clone())
            .set_redirect_uri(self.redirect_uri.clone());

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .url();

        (auth_url, pkce_verifier, csrf_token)
    }

    /// Exchanges an authorization code (from the redirect after `authorization_url`) for an
    /// access/refresh token pair. `verifier` is the one returned alongside that URL.
    pub async fn exchange_code(
        &self,
        code: impl Into<String>,
        verifier: PkceCodeVerifier,
    ) -> Result<OAuthTokens, Error> {
        let client = BasicClient::new(self.client_id.clone())
            .set_client_secret(self.client_secret.clone())
            .set_auth_uri(self.auth_url.clone())
            .set_token_uri(self.token_url.clone())
            .set_redirect_uri(self.redirect_uri.clone());

        let response = client
            .exchange_code(AuthorizationCode::new(code.into()))
            .set_pkce_verifier(verifier)
            .request_async(&|req| self.http_response(req))
            .await
            .map_err(|e| Error::OAuth(format!("{e:?}")))?;

        Ok(tokens_from_response(&response))
    }

    /// Exchanges a refresh token for a new access token. If the response doesn't include a new
    /// refresh token, keep using the one you already have.
    pub async fn refresh_token(
        &self,
        refresh_token: impl Into<String>,
    ) -> Result<OAuthTokens, Error> {
        let client = BasicClient::new(self.client_id.clone())
            .set_client_secret(self.client_secret.clone())
            .set_auth_uri(self.auth_url.clone())
            .set_token_uri(self.token_url.clone())
            .set_redirect_uri(self.redirect_uri.clone());

        let refresh_token = RefreshToken::new(refresh_token.into());
        let response = client
            .exchange_refresh_token(&refresh_token)
            .request_async(&|req| self.http_response(req))
            .await
            .map_err(|e| Error::OAuth(format!("{e:?}")))?;

        Ok(tokens_from_response(&response))
    }

    /// Parses the full redirect URL your callback handler received (e.g. `https://your-app/callback
    /// ?code=...&state=...`), verifies its `state` param matches the `CsrfToken` returned by
    /// `authorization_url`, and returns `code` — or an error if `state` is missing, doesn't match, or
    /// `code` itself is missing. Making this check mandatory here (rather than leaving it as a step a
    /// caller could forget) is the point: skipping CSRF verification on this flow is a real
    /// vulnerability, not just a formality.
    pub fn verify_and_extract_code(
        redirect_url: &str,
        csrf_token: &CsrfToken,
    ) -> Result<String, Error> {
        let url = Url::parse(redirect_url)?;

        let state = url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.into_owned())
            .ok_or_else(|| Error::OAuth("redirect URL is missing the `state` param".to_string()))?;

        if &state != csrf_token.secret() {
            return Err(Error::OAuth(
                "`state` param did not match the expected CSRF token".to_string(),
            ));
        }

        url.query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value.into_owned())
            .ok_or_else(|| Error::OAuth("redirect URL is missing the `code` param".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config(server: &MockServer) -> OAuthConfig {
        OAuthConfig::new(
            "test-client-id",
            "test-client-secret",
            "http://example.com/authorize",
            format!("{}/token", server.uri()),
            "http://localhost:8080/callback",
        )
        .unwrap()
    }

    #[test]
    fn verify_and_extract_code_succeeds_when_state_matches() {
        let csrf_token = CsrfToken::new("expected-state".to_string());
        let redirect_url = "https://your-app.example.com/callback?code=abc123&state=expected-state";

        let code = OAuthConfig::verify_and_extract_code(redirect_url, &csrf_token).unwrap();

        assert_eq!(code, "abc123");
    }

    #[test]
    fn verify_and_extract_code_rejects_mismatched_state() {
        let csrf_token = CsrfToken::new("expected-state".to_string());
        let redirect_url = "https://your-app.example.com/callback?code=abc123&state=wrong-state";

        let result = OAuthConfig::verify_and_extract_code(redirect_url, &csrf_token);

        assert!(matches!(result, Err(Error::OAuth(_))));
    }

    #[test]
    fn verify_and_extract_code_rejects_missing_state() {
        let csrf_token = CsrfToken::new("expected-state".to_string());
        let redirect_url = "https://your-app.example.com/callback?code=abc123";

        let result = OAuthConfig::verify_and_extract_code(redirect_url, &csrf_token);

        assert!(matches!(result, Err(Error::OAuth(_))));
    }

    #[test]
    fn verify_and_extract_code_rejects_missing_code() {
        let csrf_token = CsrfToken::new("expected-state".to_string());
        let redirect_url = "https://your-app.example.com/callback?state=expected-state";

        let result = OAuthConfig::verify_and_extract_code(redirect_url, &csrf_token);

        assert!(matches!(result, Err(Error::OAuth(_))));
    }

    fn verifier() -> PkceCodeVerifier {
        let (_challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        verifier
    }

    #[tokio::test]
    async fn exchange_code_returns_tokens() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "test-access-token",
                "token_type": "bearer",
                "refresh_token": "test-refresh-token",
                "expires_in": 7200
            })))
            .expect(1)
            .mount(&server)
            .await;

        let config = test_config(&server);
        let tokens = config.exchange_code("test-code", verifier()).await.unwrap();

        assert_eq!(tokens.access_token, "test-access-token");
        assert_eq!(tokens.refresh_token.as_deref(), Some("test-refresh-token"));
        let expires_at = tokens.expires_at.expect("expires_at should be set");
        let expected = Utc::now() + chrono::Duration::seconds(7200);
        assert!((expires_at - expected).num_seconds().abs() < 5);
    }

    #[tokio::test]
    async fn exchange_code_handles_missing_refresh_token_and_expiry() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "test-access-token",
                "token_type": "bearer"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let config = test_config(&server);
        let tokens = config.exchange_code("test-code", verifier()).await.unwrap();

        assert_eq!(tokens.access_token, "test-access-token");
        assert_eq!(tokens.refresh_token, None);
        assert_eq!(tokens.expires_at, None);
    }

    #[tokio::test]
    async fn exchange_code_returns_oauth_error_on_failure() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                "error": "invalid_grant",
                "error_description": "The code passed is incorrect or expired."
            })))
            .expect(1)
            .mount(&server)
            .await;

        let config = test_config(&server);
        let result = config.exchange_code("test-code", verifier()).await;

        assert!(matches!(result, Err(Error::OAuth(_))));
    }

    #[tokio::test]
    async fn refresh_token_returns_tokens() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            // no refresh token in response
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "new-access-token",
                "token_type": "bearer",
                "expires_in": 3600
            })))
            .expect(1)
            .mount(&server)
            .await;

        let config = test_config(&server);
        // since no refresh token present, should return original access token.
        let tokens = config.refresh_token("test-refresh-token").await.unwrap();

        assert_eq!(tokens.access_token, "new-access-token");
        assert_eq!(tokens.refresh_token, None);
        assert!(tokens.expires_at.is_some());
    }
}
