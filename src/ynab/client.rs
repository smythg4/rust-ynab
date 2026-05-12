use crate::ynab::errors::{Error, ErrorResponse};
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use secrecy::ExposeSecret;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
/// Client is the YNAB API client. Use Client::new() to create one.
pub struct Client {
    base_url: reqwest::Url,
    http_client: reqwest::Client,
    limiter: Option<Arc<DefaultDirectRateLimiter>>,
    api_key: secrecy::SecretBox<String>,
    timeout: Option<Duration>,
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
        let http_client = Self::build_http_client(api_key.expose_secret(), None)?;
        Ok(Self {
            base_url: reqwest::Url::parse("https://api.ynab.com/v1")
                .expect("hardcoded base URL is always valid"),
            http_client,
            limiter: None,
            api_key,
            timeout: None,
        })
    }

    fn build_http_client(
        api_key: &str,
        timeout: Option<Duration>,
    ) -> Result<reqwest::Client, Error> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", api_key)
                .parse()
                .expect("api key must be valid ASCII"),
        );
        let mut builder = reqwest::Client::builder().default_headers(headers);
        if let Some(t) = timeout {
            builder = builder.timeout(t);
        }
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
        self.http_client = Self::build_http_client(self.api_key.expose_secret(), Some(timeout))?;
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

    pub(crate) async fn get<T: serde::de::DeserializeOwned, Q: serde::ser::Serialize + ?Sized>(
        &self,
        endpoint: &str,
        params: Option<&Q>,
    ) -> Result<T, Error> {
        if let Some(limiter) = &self.limiter {
            limiter.until_ready().await;
        }

        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("base URL must be a valid base")
            .extend(endpoint.split('/'));

        let mut builder = self.http_client.get(url);
        if let Some(p) = params {
            builder = builder.query(p);
        }
        let res = builder.send().await?;
        let status = res.status();

        if !status.is_success() {
            let err_body: ErrorResponse = res.json().await?;
            return Err(Error::new_api_error(status, err_body.error));
        }

        res.json().await.map_err(Into::into)
    }

    pub(crate) async fn post<T: serde::de::DeserializeOwned, B: serde::ser::Serialize>(
        &self,
        endpoint: &str,
        body: B,
    ) -> Result<T, Error> {
        if let Some(limiter) = &self.limiter {
            limiter.until_ready().await;
        }
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("base URL must be a valid base")
            .extend(endpoint.split('/'));

        let res = self.http_client.post(url).json(&body).send().await?;
        let status = res.status();

        if !status.is_success() {
            let err_body: ErrorResponse = res.json().await?;
            return Err(Error::new_api_error(status, err_body.error));
        }

        res.json().await.map_err(Into::into)
    }

    pub(crate) async fn patch<T: serde::de::DeserializeOwned, B: serde::ser::Serialize>(
        &self,
        endpoint: &str,
        body: B,
    ) -> Result<T, Error> {
        if let Some(limiter) = &self.limiter {
            limiter.until_ready().await;
        }
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("base URL must be a valid base")
            .extend(endpoint.split('/'));

        let res = self.http_client.patch(url).json(&body).send().await?;
        let status = res.status();

        if !status.is_success() {
            let err_body: ErrorResponse = res.json().await?;
            return Err(Error::new_api_error(status, err_body.error));
        }

        res.json().await.map_err(Into::into)
    }

    pub(crate) async fn put<T: serde::de::DeserializeOwned, B: serde::ser::Serialize>(
        &self,
        endpoint: &str,
        body: B,
    ) -> Result<T, Error> {
        if let Some(limiter) = &self.limiter {
            limiter.until_ready().await;
        }
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("base URL must be a valid base")
            .extend(endpoint.split('/'));

        let res = self.http_client.put(url).json(&body).send().await?;
        let status = res.status();

        if !status.is_success() {
            let err_body: ErrorResponse = res.json().await?;
            return Err(Error::new_api_error(status, err_body.error));
        }

        res.json().await.map_err(Into::into)
    }

    pub(crate) async fn delete<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<T, Error> {
        if let Some(limiter) = &self.limiter {
            limiter.until_ready().await;
        }
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("base URL must be a valid base")
            .extend(endpoint.split('/'));

        let res = self.http_client.delete(url).send().await?;
        let status = res.status();

        if !status.is_success() {
            let err_body: ErrorResponse = res.json().await?;
            return Err(Error::new_api_error(status, err_body.error));
        }

        res.json().await.map_err(Into::into)
    }
}
