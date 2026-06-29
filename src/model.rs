use serde::{Deserialize, Serialize};

pub const VECTOR_DIMENSIONS: usize = 14;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FraudScoreRequest {
    pub id: String,
    pub transaction: Transaction,
    pub customer: Customer,
    pub merchant: Merchant,
    pub terminal: Terminal,
    pub last_transaction: Option<LastTransaction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub amount: f64,
    pub installments: u32,
    pub requested_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Customer {
    pub avg_amount: f64,
    pub tx_count_24h: u32,
    pub known_merchants: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Merchant {
    pub id: String,
    pub mcc: String,
    pub avg_amount: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Terminal {
    pub is_online: bool,
    pub card_present: bool,
    pub km_from_home: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LastTransaction {
    pub timestamp: String,
    pub km_from_current: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FraudScoreResponse {
    pub approved: bool,
    pub fraud_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyResponse {
    pub status: &'static str,
    pub references_loaded: usize,
    pub reference_source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Normalization {
    pub max_amount: f64,
    pub max_installments: f64,
    pub amount_vs_avg_ratio: f64,
    pub max_minutes: f64,
    pub max_km: f64,
    pub max_tx_count_24h: f64,
    pub max_merchant_avg_amount: f64,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Label {
    Fraud,
    Legit,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReferenceRecord {
    pub vector: Vec<f32>,
    pub label: Label,
}

#[derive(Debug, Clone)]
pub struct ReferencePoint {
    pub vector: [f32; VECTOR_DIMENSIONS],
    pub label: Label,
}
