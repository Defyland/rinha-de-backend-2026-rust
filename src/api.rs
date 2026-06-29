use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};

use crate::{
    engine::{DecisionEngine, EngineError},
    model::{ErrorResponse, FraudScoreRequest, FraudScoreResponse, ReadyResponse},
};

#[derive(Clone)]
pub struct AppState {
    engine: Arc<DecisionEngine>,
    reference_source: String,
}

pub fn router(engine: Arc<DecisionEngine>, reference_source: String) -> Router {
    Router::new()
        .route("/ready", get(ready))
        .route("/fraud-score", post(fraud_score))
        .with_state(AppState {
            engine,
            reference_source,
        })
}

async fn ready(State(state): State<AppState>) -> Json<ReadyResponse> {
    Json(ReadyResponse {
        status: "ok",
        references_loaded: state.engine.reference_count(),
        reference_source: state.reference_source,
    })
}

async fn fraud_score(
    State(state): State<AppState>,
    Json(payload): Json<FraudScoreRequest>,
) -> Result<Json<FraudScoreResponse>, ApiError> {
    Ok(Json(state.engine.score(&payload)?))
}

struct ApiError(EngineError);

impl From<EngineError> for ApiError {
    fn from(error: EngineError) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match self.0 {
            EngineError::EmptyReferenceDataset => StatusCode::SERVICE_UNAVAILABLE,
            EngineError::InvalidReferenceDimensions(_)
            | EngineError::Io(_)
            | EngineError::Json(_)
            | EngineError::Time(_) => StatusCode::UNPROCESSABLE_ENTITY,
        };

        (
            status,
            Json(ErrorResponse {
                error: self.0.to_string(),
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::{
        engine::DecisionEngine,
        model::{Label, Normalization, ReferencePoint},
    };

    use super::router;

    fn test_router() -> axum::Router {
        let engine = DecisionEngine::new(
            Normalization {
                max_amount: 10_000.0,
                max_installments: 12.0,
                amount_vs_avg_ratio: 10.0,
                max_minutes: 1_440.0,
                max_km: 1_000.0,
                max_tx_count_24h: 20.0,
                max_merchant_avg_amount: 10_000.0,
            },
            HashMap::from([("5411".to_string(), 0.15)]),
            vec![
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
            ],
        )
        .expect("engine should build");

        router(Arc::new(engine), "test".to_string())
    }

    #[tokio::test]
    async fn ready_reports_loaded_references() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn fraud_score_returns_decision_payload() {
        let body = r#"{
          "id": "tx-1329056812",
          "transaction": {
            "amount": 41.12,
            "installments": 2,
            "requested_at": "2026-03-11T18:45:53Z"
          },
          "customer": {
            "avg_amount": 82.24,
            "tx_count_24h": 3,
            "known_merchants": ["MERC-003", "MERC-016"]
          },
          "merchant": {
            "id": "MERC-016",
            "mcc": "5411",
            "avg_amount": 60.25
          },
          "terminal": {
            "is_online": false,
            "card_present": true,
            "km_from_home": 29.2331036248
          },
          "last_transaction": null
        }"#;

        let response = test_router()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/fraud-score")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");

        assert_eq!(response.status(), StatusCode::OK);

        let payload = response
            .into_body()
            .collect()
            .await
            .expect("body should collect")
            .to_bytes();
        let parsed: serde_json::Value =
            serde_json::from_slice(&payload).expect("response should be valid json");

        assert_eq!(parsed["approved"], true);
        assert_eq!(parsed["fraud_score"], 0.4);
    }
}
