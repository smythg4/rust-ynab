//! Demonstrates the tracing instrumentation on `Client` against a mock server —
//! no YNAB_TOKEN needed. Run with:
//!
//!     RUST_LOG=rust_ynab=debug cargo run --example trace_demo
use rust_ynab::{Client, PlanId};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let server = MockServer::start().await;

    let plan_json = json!({
        "id": "523e4567-e89b-12d3-a456-426614174000",
        "name": "My Budget",
        "last_modified_on": "2024-01-01T00:00:00Z",
        "first_month": "2024-01-01",
        "last_month": "2024-12-01",
        "date_format": { "format": "MM/DD/YYYY" },
        "currency_format": {
            "iso_code": "USD",
            "example_format": "123,456.78",
            "decimal_digits": 2,
            "decimal_separator": ".",
            "symbol_first": true,
            "group_separator": ",",
            "currency_symbol": "$",
            "display_symbol": true
        },
        "accounts": []
    });

    Mock::given(method("GET"))
        .and(path("/plans"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "plans": [plan_json], "default_plan": null }
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/plans/last-used"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": { "id": "404", "name": "not_found", "detail": "Plan not found" }
        })))
        .mount(&server)
        .await;

    // burst of 1, then ~1 req/sec sustained — enough to force a visible rate-limiter wait
    // on the very next call without making the demo sit around for a long time.
    let client = Client::new("fake-token")?
        .with_base_url(server.uri())?
        .with_rate_limiter(3600, Some(1))?;

    println!("--- request 1: consumes the burst, no rate-limiter wait ---");
    let _ = client.get_plans().send().await?;

    println!("--- request 2: burst exhausted, watch for the rate-limiter debug event ---");
    let _ = client.get_plans().send().await?;

    println!("--- request 3: mocked 404, watch for the warn event ---");
    match client.get_plan(PlanId::LastUsed).send().await {
        Ok(_) => unreachable!("mock always returns 404"),
        Err(e) => {
            assert!(matches!(e, rust_ynab::Error::NotFound(_)));
            println!("404 Not Found error as expected.");
        }
    }

    Ok(())
}
