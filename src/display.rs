use crate::Transaction;
use crate::milliunits_to_amount;

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}

pub fn print_transaction_table(transactions: &[Transaction]) {
    println!(
        "{:<12}  {:<20}  {:<25}  {:>13}",
        "Date", "Account", "Payee", "Amount"
    );
    println!(
        "{:<12}  {:<20}  {:<25}  {:>13}",
        "------------", "--------------------", "-------------------------", "-------------"
    );
    for tx in transactions {
        let account = tx
            .account_name
            .as_deref()
            .map(|s| truncate(s, 20))
            .unwrap_or_default();
        let payee = tx
            .payee_name
            .as_deref()
            .map(|s| truncate(s, 25))
            .unwrap_or_default();
        println!(
            "{:<12}  {:<20}  {:<25}  {:>13}",
            tx.date,
            account,
            payee,
            format!("${:.2}", milliunits_to_amount(tx.amount))
        );
    }
}
