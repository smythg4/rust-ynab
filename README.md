# rust-ynab

A Rust client for the [YNAB API](https://api.ynab.com). Requires a YNAB account and a [Personal Access Token](https://app.ynab.com/settings/developer).

## Installation

```toml
[dependencies]
rust-ynab = "0.1.0"
```

## Usage

### Authentication

All API access requires a Personal Access Token. Pass it to `Client::new`:

```rust
let client = Client::new(std::env::var("YNAB_TOKEN")?)?;
```

### Quick Start

```rust
use rust_ynab::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(std::env::var("YNAB_TOKEN")?)?;

    let plans = client.get_plans(true).await?;
    for plan in plans {
        println!("{}", plan.name);
        for acct in plan.accounts.unwrap_or_default() {
            println!("   {}", acct.name);
        }
    }

    Ok(())
}
```

## Rate Limiting

The [YNAB API](https://api.ynab.com/#rate-limiting) allows 200 requests per hour. `with_rate_limiter` enables a token bucket limiter that automatically spaces requests to stay within that limit:

```rust
let client = Client::new(std::env::var("YNAB_TOKEN")?)?.with_rate_limiter(200, Some(10))?;
```

The first argument is the request budget per hour; the second is the optional burst size — the number of requests that can be made immediately before throttling begins. To keep total consumption within YNAB's limit, the sustained rate is reduced by the burst size: `with_rate_limiter(200, Some(10))` allows 10 immediate requests, then throttles to 190 per hour. Calls block until a token is available rather than returning an error, so no retry logic is needed on the caller's side.

Rate limiting is opt-in. Omit `with_rate_limiter` for scripts or one-off tools where request volume is not a concern.

## Error Handling

Errors from the API are returned as variants of `errors::Error` and can be matched directly:

```rust
match client.get_plans(false).await {
    Err(Error::NotFound(e)) => eprintln!("not found: {}", e),
    Err(Error::Unauthorized(e)) => eprintln!("check your token: {}", e),
    Err(e) => return Err(e),
    Ok(plans) => { /* ... */ }
}
```

Available error variants: `BadRequest`, `Unauthorized`, `Forbidden`, `NotFound`, `Conflict`, `RateLimited`, `InternalServerError`, `ServiceUnavailable`.

## API Coverage

### Plans
| Method | Endpoint |
|--------|----------|
| `get_plans` | `GET /plans` |
| `get_plan_settings` | `GET /plans/{plan_id}/settings` |

### Months
| Method | Endpoint |
|--------|----------|
| `get_months` | `GET /plans/{plan_id}/months` |
| `get_month` | `GET /plans/{plan_id}/months/{month}` |

### Payees
| Method | Endpoint |
|--------|----------|
| `get_payees` | `GET /plans/{plan_id}/payees` |
| `get_payee` | `GET /plans/{plan_id}/payees/{payee_id}` |
| `get_payee_locations` | `GET /plans/{plan_id}/payee_locations` |
| `get_payee_location` | `GET /plans/{plan_id}/payee_locations/{payee_location_id}` |
| `get_payee_locations_by_payee` | `GET /plans/{plan_id}/payees/{payee_id}/payee_locations` |

## License

[MIT](LICENSE)

---

Not affiliated with YNAB. [YNAB API Terms of Service](https://api.ynab.com/#terms).
