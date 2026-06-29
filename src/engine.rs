use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
};

use flate2::read::GzDecoder;
use serde::de::DeserializeOwned;
use thiserror::Error;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    config::ResourcePaths,
    model::{
        FraudScoreRequest, FraudScoreResponse, Label, Normalization, ReferencePoint,
        ReferenceRecord, VECTOR_DIMENSIONS,
    },
};

#[derive(Debug, Clone)]
pub struct DecisionEngine {
    normalization: Normalization,
    mcc_risk: HashMap<String, f64>,
    references: Vec<ReferencePoint>,
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("failed to read resource: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse json resource: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to parse timestamp: {0}")]
    Time(#[from] time::error::Parse),
    #[error("reference vector has {0} dimensions, expected {VECTOR_DIMENSIONS}")]
    InvalidReferenceDimensions(usize),
    #[error("reference dataset is empty")]
    EmptyReferenceDataset,
}

impl DecisionEngine {
    pub fn load(paths: &ResourcePaths) -> Result<Self, EngineError> {
        let normalization = read_json_file(&paths.normalization)?;
        let mcc_risk = read_json_file(&paths.mcc_risk)?;
        let records: Vec<ReferenceRecord> = read_json_file(&paths.references)?;
        let references = records
            .into_iter()
            .map(ReferencePoint::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Self::new(normalization, mcc_risk, references)
    }

    pub fn new(
        normalization: Normalization,
        mcc_risk: HashMap<String, f64>,
        references: Vec<ReferencePoint>,
    ) -> Result<Self, EngineError> {
        if references.is_empty() {
            return Err(EngineError::EmptyReferenceDataset);
        }

        Ok(Self {
            normalization,
            mcc_risk,
            references,
        })
    }

    pub fn reference_count(&self) -> usize {
        self.references.len()
    }

    pub fn vectorize(
        &self,
        request: &FraudScoreRequest,
    ) -> Result<[f32; VECTOR_DIMENSIONS], EngineError> {
        let requested_at = parse_timestamp(&request.transaction.requested_at)?;

        let minutes_since_last_tx = request
            .last_transaction
            .as_ref()
            .map(|last_transaction| -> Result<f32, EngineError> {
                let last_timestamp = parse_timestamp(&last_transaction.timestamp)?;
                let minutes = (requested_at - last_timestamp).whole_minutes().max(0) as f64;

                Ok(clamp(minutes / self.normalization.max_minutes))
            })
            .transpose()?;

        let amount_vs_avg = if request.customer.avg_amount <= 0.0 {
            1.0
        } else {
            clamp(
                (request.transaction.amount / request.customer.avg_amount)
                    / self.normalization.amount_vs_avg_ratio,
            )
        };

        let unknown_merchant = if request
            .customer
            .known_merchants
            .iter()
            .any(|merchant_id| merchant_id == &request.merchant.id)
        {
            0.0
        } else {
            1.0
        };

        Ok([
            clamp(request.transaction.amount / self.normalization.max_amount),
            clamp(request.transaction.installments as f64 / self.normalization.max_installments),
            amount_vs_avg,
            requested_at.hour() as f32 / 23.0,
            requested_at.weekday().number_days_from_monday() as f32 / 6.0,
            minutes_since_last_tx.unwrap_or(-1.0),
            request
                .last_transaction
                .as_ref()
                .map(|last_transaction| {
                    clamp(last_transaction.km_from_current / self.normalization.max_km)
                })
                .unwrap_or(-1.0),
            clamp(request.terminal.km_from_home / self.normalization.max_km),
            clamp(request.customer.tx_count_24h as f64 / self.normalization.max_tx_count_24h),
            bool_as_unit(request.terminal.is_online),
            bool_as_unit(request.terminal.card_present),
            unknown_merchant,
            self.mcc_risk
                .get(&request.merchant.mcc)
                .copied()
                .unwrap_or(0.5) as f32,
            clamp(request.merchant.avg_amount / self.normalization.max_merchant_avg_amount),
        ])
    }

    pub fn score(&self, request: &FraudScoreRequest) -> Result<FraudScoreResponse, EngineError> {
        let query = self.vectorize(request)?;
        let k = self.references.len().min(5);
        let mut nearest_neighbors: Vec<(f64, Label)> = Vec::with_capacity(k);

        for reference in &self.references {
            let distance = squared_euclidean_distance(&query, &reference.vector);

            nearest_neighbors.push((distance, reference.label));
            nearest_neighbors.sort_by(|left, right| left.0.total_cmp(&right.0));

            if nearest_neighbors.len() > k {
                nearest_neighbors.pop();
            }
        }

        let fraud_neighbors = nearest_neighbors
            .iter()
            .filter(|(_, label)| matches!(label, Label::Fraud))
            .count();
        let fraud_score = fraud_neighbors as f64 / k as f64;

        Ok(FraudScoreResponse {
            approved: fraud_score < 0.6,
            fraud_score,
        })
    }
}

fn read_json_file<T: DeserializeOwned>(path: &std::path::Path) -> Result<T, EngineError> {
    let reader = open_reader(path)?;
    Ok(serde_json::from_reader(reader)?)
}

fn open_reader(path: &std::path::Path) -> Result<Box<dyn Read>, std::io::Error> {
    let file = File::open(path)?;

    if path.extension().and_then(|extension| extension.to_str()) == Some("gz") {
        return Ok(Box::new(GzDecoder::new(file)));
    }

    Ok(Box::new(BufReader::new(file)))
}

fn parse_timestamp(raw_timestamp: &str) -> Result<OffsetDateTime, time::error::Parse> {
    OffsetDateTime::parse(raw_timestamp, &Rfc3339)
}

fn bool_as_unit(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}

fn clamp(value: f64) -> f32 {
    value.clamp(0.0, 1.0) as f32
}

fn squared_euclidean_distance(
    left: &[f32; VECTOR_DIMENSIONS],
    right: &[f32; VECTOR_DIMENSIONS],
) -> f64 {
    left.iter()
        .zip(right.iter())
        .map(|(left_value, right_value)| {
            let delta = *left_value as f64 - *right_value as f64;
            delta * delta
        })
        .sum()
}

impl TryFrom<ReferenceRecord> for ReferencePoint {
    type Error = EngineError;

    fn try_from(record: ReferenceRecord) -> Result<Self, Self::Error> {
        let vector: [f32; VECTOR_DIMENSIONS] = record
            .vector
            .try_into()
            .map_err(|vector: Vec<f32>| EngineError::InvalidReferenceDimensions(vector.len()))?;

        Ok(Self {
            vector,
            label: record.label,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::model::{
        Customer, FraudScoreRequest, Label, LastTransaction, Merchant, Normalization,
        ReferencePoint, Terminal, Transaction,
    };

    use super::DecisionEngine;

    fn engine_with_references(references: Vec<ReferencePoint>) -> DecisionEngine {
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
            HashMap::from([
                ("5411".to_string(), 0.15),
                ("7802".to_string(), 0.75),
                ("7995".to_string(), 0.85),
            ]),
            references,
        )
        .expect("engine should build")
    }

    fn official_legit_request() -> FraudScoreRequest {
        FraudScoreRequest {
            id: "tx-1329056812".to_string(),
            transaction: Transaction {
                amount: 41.12,
                installments: 2,
                requested_at: "2026-03-11T18:45:53Z".to_string(),
            },
            customer: Customer {
                avg_amount: 82.24,
                tx_count_24h: 3,
                known_merchants: vec!["MERC-003".to_string(), "MERC-016".to_string()],
            },
            merchant: Merchant {
                id: "MERC-016".to_string(),
                mcc: "5411".to_string(),
                avg_amount: 60.25,
            },
            terminal: Terminal {
                is_online: false,
                card_present: true,
                km_from_home: 29.2331036248,
            },
            last_transaction: None,
        }
    }

    fn official_fraud_request() -> FraudScoreRequest {
        FraudScoreRequest {
            id: "tx-3330991687".to_string(),
            transaction: Transaction {
                amount: 9505.97,
                installments: 10,
                requested_at: "2026-03-14T05:15:12Z".to_string(),
            },
            customer: Customer {
                avg_amount: 81.28,
                tx_count_24h: 20,
                known_merchants: vec![
                    "MERC-008".to_string(),
                    "MERC-007".to_string(),
                    "MERC-005".to_string(),
                ],
            },
            merchant: Merchant {
                id: "MERC-068".to_string(),
                mcc: "7802".to_string(),
                avg_amount: 54.86,
            },
            terminal: Terminal {
                is_online: false,
                card_present: true,
                km_from_home: 952.27,
            },
            last_transaction: None,
        }
    }

    #[test]
    fn vectorizes_the_official_legit_example() {
        let engine = engine_with_references(vec![ReferencePoint {
            vector: [0.0; 14],
            label: Label::Legit,
        }]);

        let vector = engine
            .vectorize(&official_legit_request())
            .expect("vectorization should succeed");
        let expected = [
            0.004112,
            0.16666667,
            0.05,
            0.7826087,
            0.33333334,
            -1.0,
            -1.0,
            0.029233104,
            0.15,
            0.0,
            1.0,
            0.0,
            0.15,
            0.006025,
        ];

        for (actual, expected) in vector.iter().zip(expected.iter()) {
            assert!((actual - expected).abs() < 0.0005);
        }
    }

    #[test]
    fn vectorizes_the_official_fraud_example() {
        let engine = engine_with_references(vec![ReferencePoint {
            vector: [0.0; 14],
            label: Label::Legit,
        }]);

        let vector = engine
            .vectorize(&official_fraud_request())
            .expect("vectorization should succeed");
        let expected = [
            0.950597, 0.8333333, 1.0, 0.2173913, 0.8333333, -1.0, -1.0, 0.95227, 1.0, 0.0, 1.0,
            1.0, 0.75, 0.005486,
        ];

        for (actual, expected) in vector.iter().zip(expected.iter()) {
            assert!((actual - expected).abs() < 0.0005);
        }
    }

    #[test]
    fn scores_with_exact_knn_threshold() {
        let engine = engine_with_references(vec![
            ReferencePoint {
                vector: [0.0; 14],
                label: Label::Legit,
            },
            ReferencePoint {
                vector: [0.01; 14],
                label: Label::Legit,
            },
            ReferencePoint {
                vector: [0.02; 14],
                label: Label::Legit,
            },
            ReferencePoint {
                vector: [0.03; 14],
                label: Label::Fraud,
            },
            ReferencePoint {
                vector: [0.04; 14],
                label: Label::Fraud,
            },
        ]);

        let result = engine
            .score(&official_legit_request())
            .expect("scoring should succeed");

        assert!(result.approved);
        assert_eq!(result.fraud_score, 0.4);
    }

    #[test]
    fn uses_last_transaction_features_when_present() {
        let engine = engine_with_references(vec![ReferencePoint {
            vector: [0.0; 14],
            label: Label::Legit,
        }]);
        let mut request = official_legit_request();

        request.last_transaction = Some(LastTransaction {
            timestamp: "2026-03-11T14:58:35Z".to_string(),
            km_from_current: 18.8626479774,
        });

        let vector = engine
            .vectorize(&request)
            .expect("vectorization should succeed");

        assert!(vector[5] > 0.15);
        assert!(vector[6] > 0.01);
    }
}
