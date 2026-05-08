use crate::ynab::client::Client;
use serde_json::{Value, json};
use wiremock::MockServer;

pub const TEST_ID_1: &str = "123e4567-e89b-12d3-a456-426614174000";
pub const TEST_ID_2: &str = "223e4567-e89b-12d3-a456-426614174000";
pub const TEST_ID_3: &str = "323e4567-e89b-12d3-a456-426614174000";
pub const TEST_ID_4: &str = "423e4567-e89b-12d3-a456-426614174000";
pub const TEST_ID_5: &str = "523e4567-e89b-12d3-a456-426614174000";

pub async fn new_test_client() -> (Client, MockServer) {
    let server = MockServer::start().await;
    let client = Client::new("fake-token")
        .unwrap()
        .with_base_url(server.uri())
        .unwrap();
    (client, server)
}

pub fn user_fixture() -> Value {
    json!({ "id": TEST_ID_3 })
}

pub fn account_fixture() -> Value {
    json!({
        "id": TEST_ID_1,
        "name": "Checking",
        "type": "checking",
        "on_budget": true,
        "closed": false,
        "note": null,
        "balance": 100000,
        "cleared_balance": 95000,
        "uncleared_balance": 5000,
        "transfer_payee_id": null,
        "direct_import_linked": false,
        "direct_import_in_error": false,
        "last_reconciled_at": null,
        "deleted": false
    })
}

pub fn category_fixture() -> Value {
    json!({
        "id": TEST_ID_1,
        "category_group_id": TEST_ID_2,
        "category_group_name": "Everyday Expenses",
        "name": "Groceries",
        "hidden": false,
        "original_category_group_id": null,
        "note": null,
        "budgeted": 50000,
        "activity": -30000,
        "balance": 20000,
        "goal_type": null,
        "goal_needs_whole_amount": null,
        "goal_day": null,
        "goal_cadence": null,
        "goal_cadence_frequency": null,
        "goal_creation_month": null,
        "goal_target": null,
        "goal_target_date": null,
        "goal_target_month": null,
        "goal_percentage_complete": null,
        "goal_months_to_budget": null,
        "goal_under_funded": null,
        "goal_overall_funded": null,
        "goal_overall_left": null,
        "goal_snoozed_at": null,
        "deleted": false
    })
}

pub fn category_group_fixture() -> Value {
    json!({
        "id": TEST_ID_2,
        "name": "Everyday Expenses",
        "hidden": false,
        "deleted": false,
        "categories": [category_fixture()]
    })
}

pub fn payee_fixture() -> Value {
    json!({
        "id": TEST_ID_3,
        "name": "Amazon",
        "transfer_account_id": null,
        "deleted": false
    })
}

pub fn payee_location_fixture() -> Value {
    json!({
        "id": TEST_ID_4,
        "payee_id": TEST_ID_3,
        "latitude": "37.7749",
        "longitude": "-122.4194",
        "deleted": false
    })
}

pub fn transaction_fixture() -> Value {
    json!({
        "id": "transaction-1",
        "date": "2024-01-15",
        "amount": -50000,
        "memo": null,
        "cleared": "cleared",
        "approved": true,
        "flag_color": null,
        "flag_name": null,
        "account_id": TEST_ID_1,
        "payee_id": TEST_ID_3,
        "account_name": "Checking",
        "payee_name": "Amazon",
        "category_id": TEST_ID_1,
        "category_name": "Groceries",
        "matched_transaction_id": null,
        "subtransactions": []
    })
}

pub fn subtransaction_fixture() -> Value {
    json!({
        "id": "subtransaction-1",
        "transaction_id": "transaction-1",
        "amount": -25000,
        "memo": null,
        "payee_id": null,
        "payee_name": null,
        "category_id": TEST_ID_1,
        "category_name": "Groceries",
        "transfer_account_id": null,
        "transfer_transaction_id": null
    })
}

pub fn scheduled_transaction_fixture() -> Value {
    json!({
        "id": TEST_ID_4,
        "date_first": "2024-01-01",
        "date_next": "2024-02-01",
        "frequency": "monthly",
        "amount": -50000,
        "memo": null,
        "flag_color": null,
        "flag_name": null,
        "account_id": TEST_ID_1,
        "payee_id": TEST_ID_3,
        "category_id": TEST_ID_1,
        "account_name": "Checking",
        "payee_name": "Amazon",
        "category_name": "Groceries",
        "subtransactions": [],
        "transfer_account_id": null
    })
}

pub fn scheduled_subtransaction_fixture() -> Value {
    json!({
        "id": TEST_ID_4,
        "scheduled_transaction_id": TEST_ID_4,
        "amount": -25000,
        "memo": null,
        "payee_id": null,
        "payee_name": null,
        "category_id": TEST_ID_1,
        "category_name": "Groceries",
        "transfer_account_id": null,
        "deleted": false
    })
}

pub fn month_fixture() -> Value {
    json!({
        "month": "2024-01-01",
        "note": null,
        "income": 500000,
        "budgeted": 400000,
        "activity": -300000,
        "to_be_budgeted": 100000,
        "age_of_money": null,
        "deleted": false,
        "categories": [category_fixture()]
    })
}

fn currency_format() -> Value {
    json!({
        "iso_code": "USD",
        "example_format": "123,456.78",
        "decimal_digits": 2,
        "decimal_separator": ".",
        "symbol_first": true,
        "group_separator": ",",
        "currency_symbol": "$",
        "display_symbol": true
    })
}

pub fn plan_fixture() -> Value {
    json!({
        "id": TEST_ID_5,
        "name": "My Budget",
        "last_modified_on": "2024-01-01T00:00:00Z",
        "first_month": "2024-01-01",
        "last_month": "2024-12-01",
        "date_format": { "format": "MM/DD/YYYY" },
        "currency_format": currency_format(),
        "accounts": [account_fixture()]
    })
}

pub fn plan_details_fixture() -> Value {
    let mut v = plan_fixture();
    v["payees"] = json!([payee_fixture()]);
    v["payee_locations"] = json!([payee_location_fixture()]);
    v["category_groups"] = json!([category_group_fixture()]);
    v["categories"] = json!([category_fixture()]);
    v["months"] = json!([month_fixture()]);
    v["transactions"] = json!([transaction_fixture()]);
    v["subtransactions"] = json!([subtransaction_fixture()]);
    v["scheduled_transactions"] = json!([scheduled_transaction_fixture()]);
    v["scheduled_subtransactions"] = json!([scheduled_subtransaction_fixture()]);
    v
}

pub fn error_body(id: &str, name: &str, detail: &str) -> Value {
    json!({ "error": { "id": id, "name": name, "detail": detail } })
}
