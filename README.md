# rust-ynab

A Rust client for the [YNAB API](https://api.ynab.com). Supports full access to all published YNAB API endpoints. Requires a YNAB account and a [Personal Access Token](https://app.ynab.com/settings/developer).

## Installation

```toml
[dependencies]
rust-ynab = "0.1.0"
```

## Usage

### Authentication

All API access requires a Personal Access Token. Pass it to `Client::new`:

```rust
let client = Client::new(&std::env::var("YNAB_TOKEN")?)?;
```

### Quick Start

```rust
use rust_ynab::{Client, PlanId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(&std::env::var("YNAB_TOKEN")?)?;

    let plans = client.get_plans().include_accounts().send().await?;
    for plan in plans {
        println!("{}", plan.name);
        for acct in &plan.accounts {
            println!("   {}", acct.name);
        }
    }

    Ok(())
}
```

## Builder Pattern

Methods that support optional parameters use a builder. Call the factory method on the client, chain any options, then call `.send()`:

```rust
// fetch only changes since the last sync
let (transactions, server_knowledge) = client
    .get_transactions(PlanId::LastUsed)
    .with_server_knowledge(last_known)
    .since_date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
    .send()
    .await?;

// include sub-resources inline
let plans = client.get_plans().include_accounts().send().await?;
```

## Rate Limiting

The [YNAB API](https://api.ynab.com/#rate-limiting) allows 200 requests per hour. `with_rate_limiter` enables a token bucket limiter that automatically spaces requests to stay within that limit:

```rust
let client = Client::new(&std::env::var("YNAB_TOKEN")?)?.with_rate_limiter(200, Some(10))?;
```

The first argument is the request budget per hour; the second is the optional burst size — the number of requests that can be made immediately before throttling begins. To keep total consumption within YNAB's limit, the sustained rate is reduced by the burst size: `with_rate_limiter(200, Some(10))` allows 10 immediate requests, then throttles to 190 per hour. Calls block until a token is available rather than returning an error, so no retry logic is needed on the caller's side.

Rate limiting is opt-in. Omit `with_rate_limiter` for scripts or one-off tools where request volume is not a concern.

## Timeout

The default request timeout is determined by `reqwest`. Use `with_timeout` to override it:

```rust
use std::time::Duration;

let client = Client::new(&std::env::var("YNAB_TOKEN")?)?.with_timeout(Duration::from_secs(30))?;
```

Both `with_timeout` and `with_rate_limiter` return the client, so they can be chained:

```rust
let client = Client::new(&std::env::var("YNAB_TOKEN")?)?
    .with_rate_limiter(200, Some(10))?
    .with_timeout(Duration::from_secs(30))?;
```

## Error Handling

Errors from the API are returned as variants of `Error` and can be matched directly:

```rust
match client.get_plan(PlanId::LastUsed).send().await {
    Err(Error::NotFound(e)) => eprintln!("plan not found: {}", e),
    Err(Error::Unauthorized(e)) => eprintln!("check your token: {}", e),
    Err(e) => return Err(Box::new(e)),
    Ok((plan, _)) => { /* ... */ }
}
```

Available error variants: `BadRequest`, `Unauthorized`, `Forbidden`, `NotFound`, `Conflict`, `RateLimited`, `InternalServerError`, `ServiceUnavailable`.

## Examples

- [List plans](examples/list_plans.rs)
- [Get plan month](examples/get_plan_month.rs)
- [Get category balance](examples/get_category_balance.rs)
- [List transactions](examples/list_transactions.rs)
- [Create transaction](examples/create_transaction.rs)
- [Create multiple transactions](examples/create_transactions.rs)
- [Update transaction](examples/update_transaction.rs)
- [Update multiple transactions](examples/update_transactions.rs)
- [Update category budget](examples/update_category_budget.rs)
- [Delete transaction](examples/delete_transaction.rs)
- [Split transaction](examples/split_transaction.rs)
- [Delta request](examples/delta_request.rs)

## API Coverage

### Plans
| Method | Endpoint |
|--------|----------|
| `get_plans` | `GET /plans` |
| `get_plan` † | `GET /plans/{plan_id}` |
| `get_plan` with `PlanId::LastUsed` | `GET /plans/last-used` |
| `get_plan_settings` | `GET /plans/{plan_id}/settings` |

### Accounts
| Method | Endpoint |
|--------|----------|
| `get_accounts` † | `GET /plans/{plan_id}/accounts` |
| `get_account` | `GET /plans/{plan_id}/accounts/{account_id}` |
| `create_account` | `POST /plans/{plan_id}/accounts` |

### Categories
| Method | Endpoint |
|--------|----------|
| `get_categories` † | `GET /plans/{plan_id}/categories` |
| `get_category` | `GET /plans/{plan_id}/categories/{category_id}` |
| `get_category_for_month` | `GET /plans/{plan_id}/months/{month}/categories/{category_id}` |
| `create_category` † | `POST /plans/{plan_id}/categories` |
| `create_category_group` † | `POST /plans/{plan_id}/category_groups` |
| `update_category` † | `PATCH /plans/{plan_id}/categories/{category_id}` |
| `update_category_for_month` † | `PATCH /plans/{plan_id}/months/{month}/categories/{category_id}` |
| `update_category_group` † | `PATCH /plans/{plan_id}/category_groups/{category_group_id}` |

### Months
| Method | Endpoint |
|--------|----------|
| `get_months` † | `GET /plans/{plan_id}/months` |
| `get_month` | `GET /plans/{plan_id}/months/{month}` |

### Payees
| Method | Endpoint |
|--------|----------|
| `get_payees` † | `GET /plans/{plan_id}/payees` |
| `get_payee` | `GET /plans/{plan_id}/payees/{payee_id}` |
| `get_payee_locations` | `GET /plans/{plan_id}/payee_locations` |
| `get_payee_location` | `GET /plans/{plan_id}/payee_locations/{payee_location_id}` |
| `get_payee_locations_by_payee` | `GET /plans/{plan_id}/payees/{payee_id}/payee_locations` |
| `create_payee` † | `POST /plans/{plan_id}/payees` |
| `update_payee` † | `PATCH /plans/{plan_id}/payees/{payee_id}` |

### Transactions
| Method | Endpoint |
|--------|----------|
| `get_transactions` † | `GET /plans/{plan_id}/transactions` |
| `get_transaction` † | `GET /plans/{plan_id}/transactions/{transaction_id}` |
| `get_transactions_by_account` † | `GET /plans/{plan_id}/accounts/{account_id}/transactions` |
| `get_transactions_by_category` † | `GET /plans/{plan_id}/categories/{category_id}/transactions` |
| `get_transactions_by_payee` † | `GET /plans/{plan_id}/payees/{payee_id}/transactions` |
| `get_transactions_by_month` † | `GET /plans/{plan_id}/months/{month}/transactions` |
| `create_transaction` | `POST /plans/{plan_id}/transactions` |
| `create_transactions` | `POST /plans/{plan_id}/transactions` |
| `update_transaction` | `PUT /plans/{plan_id}/transactions/{transaction_id}` |
| `update_transactions` | `PATCH /plans/{plan_id}/transactions` |
| `delete_transaction` † | `DELETE /plans/{plan_id}/transactions/{transaction_id}` |
| `import_transactions` | `POST /plans/{plan_id}/transactions/import` |

### Scheduled Transactions
| Method | Endpoint |
|--------|----------|
| `get_scheduled_transactions` † | `GET /plans/{plan_id}/scheduled_transactions` |
| `get_scheduled_transaction` | `GET /plans/{plan_id}/scheduled_transactions/{scheduled_transaction_id}` |
| `create_scheduled_transaction` | `POST /plans/{plan_id}/scheduled_transactions` |
| `update_scheduled_transaction` | `PUT /plans/{plan_id}/scheduled_transactions/{scheduled_transaction_id}` |
| `delete_scheduled_transaction` | `DELETE /plans/{plan_id}/scheduled_transactions/{scheduled_transaction_id}` |

### Money Movements
| Method | Endpoint |
|--------|----------|
| `get_money_movements` † | `GET /plans/{plan_id}/money_movements` |
| `get_money_movements_by_month` † | `GET /plans/{plan_id}/months/{month}/money_movements` |
| `get_money_movement_groups` † | `GET /plans/{plan_id}/money_movement_groups` |
| `get_money_movement_groups_by_month` † | `GET /plans/{plan_id}/months/{month}/money_movement_groups` |

### User
| Method | Endpoint |
|--------|----------|
| `get_user` | `GET /user` |

† Returns server knowledge as a second return value for use with delta requests.

## Test Coverage

Unit tests use [wiremock](https://github.com/LukeMathWalker/wiremock-rs) to cover all endpoints (GET, POST, PATCH, PUT, DELETE), client configuration, error type dispatch, and auth header injection. Write operation tests verify the HTTP method and request body serialization.

```
cargo test
```

Integration tests exercise the live API against a real plan and require `YNAB_TOKEN` and `YNAB_TEST_PLAN_ID` environment variables. They are opt-in via a feature flag:

```
YNAB_TOKEN=... YNAB_TEST_PLAN_ID=... cargo test --features integration
```

## License

[MIT](LICENSE)

---

Not affiliated with YNAB. [YNAB API Terms of Service](https://api.ynab.com/#terms).
