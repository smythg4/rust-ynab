use crate::ynab::errors::{Error, ErrorResponse};
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use reqwest::RequestBuilder;
use secrecy::ExposeSecret;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// Controls automatic retry of transient failures. See [`Client::with_retry`].
#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

#[derive(Debug)]
/// Client is the YNAB API client. Use Client::new() to create one.
pub struct Client {
    pub(crate) base_url: reqwest::Url,
    pub(crate) http_client: reqwest::Client,
    pub(crate) limiter: Option<Arc<DefaultDirectRateLimiter>>,
    #[allow(dead_code)]
    api_key: secrecy::SecretBox<String>, // in case we need to use this later on
    pub(crate) timeout: Option<Duration>,
    pub(crate) retry_policy: Option<RetryPolicy>,
}

impl Client {
    /// Creates a new client with the given Personal Access Token.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_ynab::Client;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new(&std::env::var("YNAB_TOKEN")?)?;
    /// # Ok(()) }
    /// ```
    pub fn new(api_key: impl Into<String>) -> Result<Self, Error> {
        let api_key = secrecy::SecretBox::new(Box::new(api_key.into()));
        let http_client = Self::build_http_client(api_key.expose_secret())?;
        Ok(Self {
            base_url: reqwest::Url::parse("https://api.ynab.com/v1")
                .expect("hardcoded base URL is always valid"),
            http_client,
            limiter: None,
            api_key,
            timeout: None,
            retry_policy: None,
        })
    }

    fn build_http_client(api_key: &str) -> Result<reqwest::Client, Error> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", api_key)
                .parse()
                .expect("api key must be valid ASCII"),
        );
        let builder = reqwest::Client::builder().default_headers(headers);
        builder.build().map_err(Into::into)
    }

    /// Sets the request timeout. Returns `self` for chaining.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_ynab::Client;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new(&std::env::var("YNAB_TOKEN")?)?
    ///     .with_timeout(Duration::from_secs(30))?;
    /// # Ok(()) }
    /// ```
    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self, Error> {
        self.timeout = Some(timeout);
        Ok(self)
    }

    /// Overrides the base URL. Primarily useful for testing.
    pub fn with_base_url(mut self, base_url: impl AsRef<str>) -> Result<Self, Error> {
        self.base_url = reqwest::Url::parse(base_url.as_ref())?;
        Ok(self)
    }

    /// Configures a token bucket rate limiter on the client. Returns `self` for chaining.
    ///
    /// `requests_per_hour` is the total allowed requests per hour.
    /// `burst_volume` optionally allows a number of requests to be made immediately
    /// before throttling begins. The effective sustained rate becomes
    /// `requests_per_hour - burst_volume` to account for burst consumption.
    /// If `None`, no burst is allowed and the full rate is sustained evenly.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_ynab::Client;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new(&std::env::var("YNAB_TOKEN")?)?
    ///     .with_rate_limiter(200, Some(10))?  // 10 burst, then 190/hr
    ///     .with_timeout(Duration::from_secs(30))?;
    /// # Ok(()) }
    /// ```
    pub fn with_rate_limiter(
        mut self,
        requests_per_hour: usize,
        burst_volume: Option<usize>,
    ) -> Result<Self, Error> {
        let requests = NonZeroU32::new(requests_per_hour as u32)
            .ok_or_else(|| Error::InvalidRateLimit("requests_per_hour must be non-zero".into()))?;

        let quota = match burst_volume {
            None => Quota::per_hour(requests),
            Some(burst) => {
                let effective = (requests_per_hour as u32)
                    .checked_sub(burst as u32)
                    .ok_or_else(|| {
                        Error::InvalidRateLimit(
                            "requests_per_hour must be greater than burst_volume".into(),
                        )
                    })?;
                let effective_rate = NonZeroU32::new(effective).ok_or_else(|| {
                    Error::InvalidRateLimit(
                        "requests_per_hour - burst_volume must be non-zero".into(),
                    )
                })?;
                let burst = NonZeroU32::new(burst as u32).ok_or_else(|| {
                    Error::InvalidRateLimit("burst_volume must be non-zero".into())
                })?;
                Quota::per_hour(effective_rate).allow_burst(burst)
            }
        };

        self.limiter = Some(Arc::new(RateLimiter::direct(quota)));
        Ok(self)
    }

    /// Enables automatic retry of transient failures (429, 503, and — for `GET` requests only —
    /// connection-level errors). Returns `self` for chaining.
    ///
    /// A `GET` is always safe to retry. A write (`POST`/`PUT`/`PATCH`/`DELETE`) is only retried
    /// when a response was actually received with status 429 or 503 — that means YNAB rejected
    /// the request before processing it. A write is never retried on a connection error or
    /// timeout, since it's impossible to tell whether the server already applied it before the
    /// connection dropped; retrying blind there risks creating a duplicate transaction.
    ///
    /// If the response carries a `Retry-After` header, that delay is used as-is. Otherwise the
    /// wait is `base_delay * 2^attempt`, capped at `max_delay`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_ynab::Client;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new(&std::env::var("YNAB_TOKEN")?)?
    ///     .with_retry(3, Duration::from_millis(200), Duration::from_secs(10));
    /// # Ok(()) }
    /// ```
    pub fn with_retry(
        mut self,
        max_retries: u32,
        base_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        self.retry_policy = Some(RetryPolicy {
            max_retries,
            base_delay,
            max_delay,
        });
        self
    }

    /// Returns the delay before the next retry attempt, or `None` if no retry policy is
    /// configured or the retry budget for this request is exhausted.
    fn next_retry_delay(&self, attempt: u32) -> Option<Duration> {
        let policy = self.retry_policy?;
        if attempt >= policy.max_retries {
            return None;
        }
        let exp = policy
            .base_delay
            .saturating_mul(2u32.saturating_pow(attempt));
        Some(exp.min(policy.max_delay))
    }

    pub(crate) fn make_url(&self, endpoint: &str) -> reqwest::Url {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("base URL must be a valid base")
            .extend(endpoint.split('/').filter(|s| !s.is_empty()));
        url
    }

    // shared: rate-limit wait + URL build + method selection
    async fn prepare(&self, method: reqwest::Method, endpoint: &str) -> RequestBuilder {
        if let Some(limiter) = &self.limiter {
            let start = std::time::Instant::now();
            limiter.until_ready().await;
            tracing::debug!(
                waited_ms = start.elapsed().as_millis() as u64,
                "rate limiter delayed request"
            );
        }
        let url = self.make_url(endpoint);
        self.http_client.request(method, url)
    }

    // shared: timeout + send + status check + error decode + body decode + retry
    async fn send_json<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        mut builder: RequestBuilder,
    ) -> Result<T, Error> {
        if let Some(t) = self.timeout {
            builder = builder.timeout(t);
        }
        let mut attempt: u32 = 0;
        loop {
            let start = std::time::Instant::now();
            // try_clone should never fail since we aren't building a stream
            let req = builder
                .try_clone()
                .expect("body must be clonable for retry");
            let res = match req.send().await {
                Ok(res) => res,
                Err(e) => {
                    // a write that never got a response might already have landed server-side —
                    // only a GET is safe to blindly retry on a connection-level failure.
                    if method == reqwest::Method::GET
                        && let Some(delay) = self.next_retry_delay(attempt)
                    {
                        tracing::debug!(
                            attempt,
                            delay_ms = delay.as_millis() as u64,
                            error = %e,
                            "retrying after connection error"
                        );
                        tokio::time::sleep(delay).await;
                        attempt += 1;
                        continue;
                    }
                    return Err(e.into());
                }
            };
            let status = res.status();
            let elapsed_ms = start.elapsed().as_millis() as u64;

            if !status.is_success() {
                let retry_after = res
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(Duration::from_secs);
                let err_body: ErrorResponse = res.json().await?;
                tracing::warn!(status = status.as_u16(), elapsed_ms, error = %err_body.error, "YNAB API error");

                // 429/503 mean the request was rejected, not processed — safe to retry
                // regardless of method, unlike a connection-level failure.
                let retryable_status = matches!(
                    status,
                    reqwest::StatusCode::TOO_MANY_REQUESTS
                        | reqwest::StatusCode::SERVICE_UNAVAILABLE
                );
                if retryable_status && let Some(backoff) = self.next_retry_delay(attempt) {
                    let delay = retry_after.unwrap_or(backoff);
                    tracing::debug!(
                        attempt,
                        delay_ms = delay.as_millis() as u64,
                        status = status.as_u16(),
                        "retrying request"
                    );
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                    continue;
                }
                return Err(Error::new_api_error(status, err_body.error));
            }
            tracing::debug!(status = status.as_u16(), elapsed_ms, "request succeeded");
            return res.json().await.map_err(Into::into);
        }
    }

    #[tracing::instrument(skip(self, params), fields(endpoint = %endpoint))]
    pub(crate) async fn get<T: serde::de::DeserializeOwned, Q: serde::ser::Serialize + ?Sized>(
        &self,
        endpoint: &str,
        params: Option<&Q>,
    ) -> Result<T, Error> {
        let mut builder = self.prepare(reqwest::Method::GET, endpoint).await;
        if let Some(p) = params {
            builder = builder.query(p);
        }
        self.send_json(reqwest::Method::GET, builder).await
    }

    #[tracing::instrument(skip(self, body), fields(endpoint = %endpoint))]
    pub(crate) async fn post<T: serde::de::DeserializeOwned, B: serde::ser::Serialize>(
        &self,
        endpoint: &str,
        body: B,
    ) -> Result<T, Error> {
        let builder = self
            .prepare(reqwest::Method::POST, endpoint)
            .await
            .json(&body);
        self.send_json(reqwest::Method::POST, builder).await
    }

    #[tracing::instrument(skip(self, body), fields(endpoint = %endpoint))]
    pub(crate) async fn patch<T: serde::de::DeserializeOwned, B: serde::ser::Serialize>(
        &self,
        endpoint: &str,
        body: B,
    ) -> Result<T, Error> {
        let builder = self
            .prepare(reqwest::Method::PATCH, endpoint)
            .await
            .json(&body);
        self.send_json(reqwest::Method::PATCH, builder).await
    }

    #[tracing::instrument(skip(self, body), fields(endpoint = %endpoint))]
    pub(crate) async fn put<T: serde::de::DeserializeOwned, B: serde::ser::Serialize>(
        &self,
        endpoint: &str,
        body: B,
    ) -> Result<T, Error> {
        let builder = self
            .prepare(reqwest::Method::PUT, endpoint)
            .await
            .json(&body);
        self.send_json(reqwest::Method::PUT, builder).await
    }

    #[tracing::instrument(skip(self), fields(endpoint = %endpoint))]
    pub(crate) async fn delete<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<T, Error> {
        let builder = self.prepare(reqwest::Method::DELETE, endpoint).await;
        self.send_json(reqwest::Method::DELETE, builder).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ynab::common::NO_PARAMS;
    use crate::ynab::testutil::{error_body, new_test_client};
    use serde::Deserialize;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    #[derive(Debug, Deserialize)]
    struct Pong {
        ok: bool,
    }

    fn too_many_requests_body() -> serde_json::Value {
        error_body("429", "too_many_requests", "Too many requests")
    }

    #[tokio::test]
    async fn retries_on_429_then_succeeds() {
        let (client, server) = new_test_client().await;
        let client = client.with_retry(3, Duration::from_millis(5), Duration::from_millis(50));

        Mock::given(method("GET"))
            .and(path("/plans"))
            .respond_with(ResponseTemplate::new(429).set_body_json(too_many_requests_body()))
            .up_to_n_times(2)
            .expect(2)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/plans"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .expect(1)
            .mount(&server)
            .await;

        let result: Pong = client.get("plans", NO_PARAMS).await.unwrap();
        assert!(result.ok);
    }

    #[tokio::test]
    async fn stops_retrying_once_budget_exhausted() {
        let (client, server) = new_test_client().await;
        let client = client.with_retry(2, Duration::from_millis(5), Duration::from_millis(50));

        Mock::given(method("GET"))
            .and(path("/plans"))
            .respond_with(ResponseTemplate::new(429).set_body_json(too_many_requests_body()))
            // initial attempt + 2 retries = 3 total requests
            .expect(3)
            .mount(&server)
            .await;

        let result: Result<Pong, Error> = client.get("plans", NO_PARAMS).await;
        assert!(matches!(result, Err(Error::RateLimited(_))));
    }

    #[tokio::test]
    async fn no_retry_without_policy_configured() {
        let (client, server) = new_test_client().await;

        Mock::given(method("GET"))
            .and(path("/plans"))
            .respond_with(ResponseTemplate::new(429).set_body_json(too_many_requests_body()))
            .expect(1)
            .mount(&server)
            .await;

        let result: Result<Pong, Error> = client.get("plans", NO_PARAMS).await;
        assert!(matches!(result, Err(Error::RateLimited(_))));
    }
}
