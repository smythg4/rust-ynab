#[cfg(feature = "integration")]
use rust_ynab::Client;
#[cfg(feature = "integration")]
use rust_ynab::PlanId;
#[cfg(feature = "integration")]
use rust_ynab::{
    ExistingTransaction, Frequency, NewTransaction, SaveScheduledTransaction, SaveSubTransaction,
    SaveTransactionWithIdOrImportId,
};
#[cfg(feature = "integration")]
use rust_ynab::{NewCategory, SaveCategory, SaveCategoryGroup};
#[cfg(feature = "integration")]
use rust_ynab::{PostPayee, SavePayee};
#[cfg(feature = "integration")]
use uuid::Uuid;

#[cfg(feature = "integration")]
type GenericError = Box<dyn std::error::Error>;

#[cfg(feature = "integration")]
fn setup() -> Result<(Client, PlanId), GenericError> {
    let token = std::env::var("YNAB_TOKEN")?;
    let pid_string = std::env::var("YNAB_TEST_PLAN_ID")?;
    let plan_id = PlanId::Id(pid_string.parse()?);

    Ok((Client::new(token)?, plan_id))
}

#[cfg(feature = "integration")]
async fn first_account_id(client: &Client, plan_id: PlanId) -> Result<Uuid, GenericError> {
    let (accounts, _sk) = client.get_accounts(plan_id).send().await?;
    if accounts.is_empty() {
        return Err("no accounts found".to_string().into());
    }
    Ok(accounts.first().unwrap().id)
}

#[cfg(feature = "integration")]
#[tokio::test]
async fn get_transactions_smoke() -> Result<(), GenericError> {
    let (client, plan_id) = setup()?;
    let (txs, sk) = client.get_transactions(plan_id).send().await?;
    assert!(sk > 0, "expected server_knowledge > 0, got {}", sk);
    println!(
        "Smoke fetched {} transactions, server_knowledge: {}",
        txs.len(),
        sk
    );

    Ok(())
}

#[cfg(feature = "integration")]
#[tokio::test]
async fn get_transactions_delta_request() -> Result<(), GenericError> {
    let (client, plan_id) = setup()?;
    let account_id = first_account_id(&client, plan_id).await?;

    let (_, sk) = client.get_transactions(plan_id).send().await?;

    assert!(sk > 0, "expected server_knowledge > 0, got {}", sk);
    println!("Initial server_knowledge: {sk}");

    let new_txs = vec![
        NewTransaction {
            account_id,
            date: chrono::Local::now().date_naive(),
            amount: Some(1000),
            payee_id: None,
            payee_name: None,
            category_id: None,
            memo: Some("integration delta request test 1".to_string()),
            cleared: Some(rust_ynab::ClearedStatus::Uncleared),
            approved: Some(false),
            flag_color: None,
            import_id: None,
            subtransactions: None,
        },
        NewTransaction {
            account_id,
            date: chrono::Local::now().date_naive(),
            amount: Some(2000),
            payee_id: None,
            payee_name: None,
            category_id: None,
            memo: Some("integration delta request test 2".to_string()),
            cleared: Some(rust_ynab::ClearedStatus::Uncleared),
            approved: Some(false),
            flag_color: None,
            import_id: None,
            subtransactions: None,
        },
    ];

    let expected_len = new_txs.len();

    let created = client.create_transactions(plan_id, new_txs).await?;

    assert_eq!(
        created.transactions.len(),
        expected_len,
        "expected {} created transaction IDs, got {}",
        expected_len,
        created.transactions.len()
    );
    println!("created transactions: {:?}", created.transactions);

    let (delta, delta_sk) = client
        .get_transactions(plan_id)
        .with_server_knowledge(sk)
        .send()
        .await?;

    assert!(
        delta.len() >= expected_len,
        "expected at least {} transactions in delta response, got {}",
        expected_len,
        delta.len()
    );
    assert!(
        delta_sk > sk,
        "expected new server_knowledge to be greater than initial server_knowledge, got initial: {} and delta: {}",
        sk,
        delta_sk
    );
    let delta_ids: std::collections::HashSet<String> =
        delta.iter().map(|tx| tx.id.clone()).collect();
    for id in &created.transaction_ids {
        assert!(
            delta_ids.contains(&id.to_string()),
            "created ID {} not in delta",
            id
        );
    }

    for tx_id in &created.transaction_ids {
        println!("deleting tx: {}", tx_id);
        client
            .delete_transaction(plan_id, &tx_id.to_string())
            .await?;
    }

    Ok(())
}

#[cfg(feature = "integration")]
#[tokio::test]
async fn transaction_create_get_delete() -> Result<(), GenericError> {
    let (client, plan_id) = setup()?;
    let account_id = first_account_id(&client, plan_id).await?;

    let memo = "integration create get delete test".to_string();
    let created = client
        .create_transaction(
            plan_id,
            NewTransaction {
                account_id,
                date: chrono::Local::now().date_naive(),
                amount: Some(1000),
                memo: Some(memo.clone()),
                cleared: Some(rust_ynab::ClearedStatus::Uncleared),
                approved: Some(false),
                payee_id: None,
                payee_name: None,
                category_id: None,
                flag_color: None,
                import_id: None,
                subtransactions: None,
            },
        )
        .await?;

    let tx_id = created.transaction.id;
    println!("created transaction: {}", tx_id);

    let fetched = client.get_transaction(plan_id, &tx_id).await?;
    assert_eq!(fetched.0.id, tx_id);
    assert_eq!(fetched.0.amount, 1000);
    assert_eq!(fetched.0.memo.as_deref(), Some(memo.as_str()));
    println!("fetched transaction: {}", fetched.0.id);

    let (deleted, _) = client
        .delete_transaction(plan_id, &tx_id.to_string())
        .await?;
    assert_eq!(deleted.id, tx_id);
    println!("deleted transaction: {}", deleted.id);

    Ok(())
}

#[cfg(feature = "integration")]
#[tokio::test]
async fn transaction_create_update_delete_single() -> Result<(), GenericError> {
    let (client, plan_id) = setup()?;
    let account_id = first_account_id(&client, plan_id).await?;

    let memo = "integration create get delete test".to_string();
    let created = client
        .create_transaction(
            plan_id,
            NewTransaction {
                account_id,
                date: chrono::Local::now().date_naive(),
                amount: Some(1000),
                memo: Some(memo.clone()),
                cleared: Some(rust_ynab::ClearedStatus::Uncleared),
                approved: Some(false),
                payee_id: None,
                payee_name: None,
                category_id: None,
                flag_color: None,
                import_id: None,
                subtransactions: None,
            },
        )
        .await?;

    let tx_id = created.transaction.id;
    println!("created transaction: {}", tx_id);

    let existing = ExistingTransaction {
        account_id: Some(account_id),
        date: None,
        amount: Some(5000),
        payee_id: None,
        payee_name: None,
        category_id: None,
        memo: None,
        cleared: None,
        approved: None,
        flag_color: None,
        subtransactions: None,
    };
    let (updated, _) = client.update_transaction(plan_id, &tx_id, existing).await?;
    assert_eq!(updated.id, tx_id);
    assert_eq!(updated.amount, 5000);
    assert_eq!(updated.memo.as_deref(), Some(memo.as_str()));
    println!("updated transaction: {}", updated.id);

    let (deleted, _) = client.delete_transaction(plan_id, &tx_id).await?;
    assert_eq!(deleted.id, tx_id);
    println!("deleted transaction: {}", deleted.id);

    Ok(())
}

#[cfg(feature = "integration")]
#[tokio::test]
async fn transactions_create_batch_and_update_batch() -> Result<(), GenericError> {
    let (client, plan_id) = setup()?;
    let account_id = first_account_id(&client, plan_id).await?;

    let created = client
        .create_transactions(
            plan_id,
            vec![
                NewTransaction {
                    account_id,
                    date: chrono::Local::now().date_naive(),
                    amount: Some(500),
                    memo: Some("integration batch tx 1".to_string()),
                    cleared: Some(rust_ynab::ClearedStatus::Uncleared),
                    approved: Some(false),
                    payee_id: None,
                    payee_name: None,
                    category_id: None,
                    flag_color: None,
                    import_id: None,
                    subtransactions: None,
                },
                NewTransaction {
                    account_id,
                    date: chrono::Local::now().date_naive(),
                    amount: Some(750),
                    memo: Some("integration batch tx 2".to_string()),
                    cleared: Some(rust_ynab::ClearedStatus::Uncleared),
                    approved: Some(false),
                    payee_id: None,
                    payee_name: None,
                    category_id: None,
                    flag_color: None,
                    import_id: None,
                    subtransactions: None,
                },
            ],
        )
        .await?;
    assert_eq!(created.transaction_ids.len(), 2);

    let txs = created.transactions;
    let patches: Vec<SaveTransactionWithIdOrImportId> = txs
        .iter()
        .map(|tx| SaveTransactionWithIdOrImportId {
            id: Some(tx.id.parse().expect("transaction id is a valid UUID")),
            memo: Some(format!("{} (updated)", tx.memo.as_deref().unwrap_or(""))),
            import_id: None,
            account_id: None,
            date: None,
            amount: None,
            payee_id: None,
            payee_name: None,
            category_id: None,
            cleared: None,
            approved: None,
            flag_color: None,
            subtransactions: None,
        })
        .collect();

    let updated = client.update_transactions(plan_id, patches).await?;
    let updated_txs = updated.transactions;
    assert_eq!(updated_txs.len(), 2);
    println!("batch updated {} transactions", updated_txs.len());

    for tx_id in &created.transaction_ids {
        client
            .delete_transaction(plan_id, &tx_id.to_string())
            .await?;
    }

    Ok(())
}

#[cfg(feature = "integration")]
#[tokio::test]
async fn transaction_split() -> Result<(), GenericError> {
    let (client, plan_id) = setup()?;
    let account_id = first_account_id(&client, plan_id).await?;

    let created = client
        .create_transaction(
            plan_id,
            NewTransaction {
                account_id,
                date: chrono::Local::now().date_naive(),
                amount: Some(5000),
                memo: Some("integration split transaction".to_string()),
                cleared: Some(rust_ynab::ClearedStatus::Uncleared),
                approved: Some(false),
                payee_id: None,
                payee_name: None,
                category_id: None,
                flag_color: None,
                import_id: None,
                subtransactions: Some(vec![
                    SaveSubTransaction {
                        amount: 2000,
                        memo: Some("split leg 1".to_string()),
                        payee_id: None,
                        payee_name: None,
                        category_id: None,
                    },
                    SaveSubTransaction {
                        amount: 3000,
                        memo: Some("split leg 2".to_string()),
                        payee_id: None,
                        payee_name: None,
                        category_id: None,
                    },
                ]),
            },
        )
        .await?;

    let tx = created.transaction;
    assert_eq!(tx.subtransactions.len(), 2);
    let subtotal: i64 = tx.subtransactions.iter().map(|s| s.amount).sum();
    assert_eq!(subtotal, tx.amount);
    println!(
        "split transaction: {} subtransactions, total {}",
        tx.subtransactions.len(),
        tx.amount
    );

    client
        .delete_transaction(plan_id, &tx.id.to_string())
        .await?;

    Ok(())
}

#[cfg(feature = "integration")]
#[tokio::test]
async fn category_create_and_update() -> Result<(), GenericError> {
    let (client, plan_id) = setup()?;

    let (group, _) = client
        .create_category_group(
            plan_id,
            SaveCategoryGroup {
                name: "integration-test-group".to_string(),
            },
        )
        .await?;
    println!("created category group: {}", group.id);

    let (cat, _) = client
        .create_category(
            plan_id,
            NewCategory {
                name: "integration-test-category".to_string(),
                category_group_id: group.id,
                note: None,
                goal_target: None,
                goal_target_date: None,
                goal_needs_whole_amount: None,
            },
        )
        .await?;
    println!("created category: {}", cat.id);

    let updated_name = "integration-test-category (updated)".to_string();
    let (updated_cat, _) = client
        .update_category(
            plan_id,
            cat.id,
            SaveCategory {
                name: Some(updated_name.clone()),
                category_group_id: None,
                note: None,
                goal_target: None,
                goal_target_date: None,
                goal_needs_whole_amount: None,
            },
        )
        .await?;
    assert_eq!(updated_cat.name, updated_name);

    let (updated_group, _) = client
        .update_category_group(
            plan_id,
            group.id,
            SaveCategoryGroup {
                name: "integration-test-group (updated)".to_string(),
            },
        )
        .await?;
    assert_eq!(updated_group.name, "integration-test-group (updated)");

    Ok(())
}

#[cfg(feature = "integration")]
#[tokio::test]
async fn payee_create_and_update() -> Result<(), GenericError> {
    let (client, plan_id) = setup()?;

    let (payee, _) = client
        .create_payee(
            plan_id,
            PostPayee {
                name: "integration-test-payee".to_string(),
            },
        )
        .await?;
    println!("created payee: {}", payee.id);

    let (updated, _) = client
        .update_payee(
            plan_id,
            payee.id,
            SavePayee {
                name: Some("integration-test-payee (updated)".to_string()),
            },
        )
        .await?;
    assert_eq!(updated.name, "integration-test-payee (updated)");

    Ok(())
}

#[cfg(feature = "integration")]
#[tokio::test]
async fn scheduled_transaction_crud() -> Result<(), GenericError> {
    let (client, plan_id) = setup()?;
    let account_id = first_account_id(&client, plan_id).await?;

    let created = client
        .create_scheduled_transaction(
            plan_id,
            SaveScheduledTransaction {
                account_id,
                date: chrono::Local::now().date_naive(),
                amount: Some(-5000),
                frequency: Some(Frequency::Monthly),
                memo: Some("integration scheduled transaction".to_string()),
                payee_id: None,
                payee_name: None,
                category_id: None,
                flag_color: None,
            },
        )
        .await?;
    println!("created scheduled transaction: {}", created.id);

    let fetched = client
        .get_scheduled_transaction(plan_id, created.id)
        .await?;
    assert_eq!(fetched.id, created.id);
    assert!(matches!(fetched.frequency, Frequency::Monthly));

    let updated = client
        .update_scheduled_transaction(
            plan_id,
            created.id,
            SaveScheduledTransaction {
                account_id,
                date: chrono::Local::now().date_naive(),
                amount: Some(-7500),
                frequency: Some(Frequency::Weekly),
                memo: Some("integration scheduled transaction (updated)".to_string()),
                payee_id: None,
                payee_name: None,
                category_id: None,
                flag_color: None,
            },
        )
        .await?;
    assert_eq!(updated.amount, -7500);
    assert!(matches!(updated.frequency, Frequency::Weekly));

    client
        .delete_scheduled_transaction(plan_id, created.id)
        .await?;
    println!("deleted scheduled transaction: {}", created.id);

    Ok(())
}
