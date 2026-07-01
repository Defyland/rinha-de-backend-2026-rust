use std::collections::HashMap;

use proptest::prelude::*;
use rinha_de_backend_2026_rust::{
    engine::DecisionEngine,
    model::{
        Customer, FraudScoreRequest, Label, LastTransaction, Merchant, Normalization,
        ReferencePoint, Terminal, Transaction,
    },
};

fn engine() -> DecisionEngine {
    DecisionEngine::new(
        Normalization {
            max_amount: 10_000.0,
            max_installments: 12.0,
            amount_vs_avg_ratio: 10.0,
            max_minutes: 1_440.0,
            max_km: 1_000.0,
            max_tx_count_24h: 20.0,
            max_merchant_avg_amount: 10_000.0,
        },
        HashMap::from([("5411".to_string(), 0.15), ("7802".to_string(), 0.75)]),
        vec![ReferencePoint {
            vector: [0.0; 14],
            label: Label::Legit,
        }],
    )
    .expect("engine should build")
}

fn request_without_last_transaction(
    amount: f64,
    installments: u32,
    customer_avg: f64,
    tx_count_24h: u32,
    merchant_avg: f64,
    km_from_home: f64,
) -> FraudScoreRequest {
    FraudScoreRequest {
        id: "tx-generated".to_string(),
        transaction: Transaction {
            amount,
            installments,
            requested_at: "2026-03-11T18:45:53Z".to_string(),
        },
        customer: Customer {
            avg_amount: customer_avg,
            tx_count_24h,
            known_merchants: vec![],
        },
        merchant: Merchant {
            id: "MERC-999".to_string(),
            mcc: "9999".to_string(),
            avg_amount: merchant_avg,
        },
        terminal: Terminal {
            is_online: false,
            card_present: true,
            km_from_home,
        },
        last_transaction: None,
    }
}

fn request_with_last_transaction(
    amount: f64,
    installments: u32,
    customer_avg: f64,
    tx_count_24h: u32,
    merchant_avg: f64,
    km_from_home: f64,
    km_from_current: f64,
) -> FraudScoreRequest {
    FraudScoreRequest {
        last_transaction: Some(LastTransaction {
            timestamp: "2026-03-11T14:58:35Z".to_string(),
            km_from_current,
        }),
        ..request_without_last_transaction(
            amount,
            installments,
            customer_avg,
            tx_count_24h,
            merchant_avg,
            km_from_home,
        )
    }
}

fn assert_normalized(
    vector: &[f32; 14],
    skip_indices: &[usize],
) -> Result<(), proptest::test_runner::TestCaseError> {
    for (index, value) in vector.iter().enumerate() {
        if skip_indices.contains(&index) {
            continue;
        }

        prop_assert!(
            (0.0..=1.0).contains(value),
            "index {index} was out of normalized range: {value}"
        );
    }

    Ok(())
}

proptest! {
    #[test]
    fn absent_last_transaction_keeps_sentinels_and_bounded_dimensions(
        amount in 0.0f64..50_000.0,
        installments in 0u32..40,
        customer_avg in 0.0f64..20_000.0,
        tx_count_24h in 0u32..100,
        merchant_avg in 0.0f64..20_000.0,
        km_from_home in 0.0f64..5_000.0,
    ) {
        let vector = engine()
            .vectorize(&request_without_last_transaction(
                amount,
                installments,
                customer_avg,
                tx_count_24h,
                merchant_avg,
                km_from_home,
            ))
            .expect("vectorization should succeed");

        prop_assert_eq!(vector[5], -1.0);
        prop_assert_eq!(vector[6], -1.0);
        prop_assert_eq!(vector[11], 1.0);
        prop_assert_eq!(vector[12], 0.5);
        assert_normalized(&vector, &[5, 6])?;
    }

    #[test]
    fn present_last_transaction_keeps_all_dimensions_bounded(
        amount in 0.0f64..50_000.0,
        installments in 0u32..40,
        customer_avg in 0.0f64..20_000.0,
        tx_count_24h in 0u32..100,
        merchant_avg in 0.0f64..20_000.0,
        km_from_home in 0.0f64..5_000.0,
        km_from_current in 0.0f64..5_000.0,
    ) {
        let vector = engine()
            .vectorize(&request_with_last_transaction(
                amount,
                installments,
                customer_avg,
                tx_count_24h,
                merchant_avg,
                km_from_home,
                km_from_current,
            ))
            .expect("vectorization should succeed");

        assert_normalized(&vector, &[])?;
    }
}
