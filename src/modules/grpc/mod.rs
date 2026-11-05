pub mod inventory_proto {
    tonic::include_proto!("inventory");
}

pub use inventory_proto::*;

use inventory_proto::inventory_service_client::InventoryServiceClient;
use tonic::transport::Channel;
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::pkg::auth::hmac::generate_hmac;
use crate::pkg::errors::AppError;

#[derive(Clone)]
pub struct InventoryClient {
    grpc_url: String,
    http_url: String,
    app_secret: String,
}

impl InventoryClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            grpc_url: if config.inventory_grpc_url.starts_with("http") {
                config.inventory_grpc_url.clone()
            } else {
                format!("http://{}", config.inventory_grpc_url)
            },
            http_url: config.inventory_url.clone(),
            app_secret: config.app_secret.clone(),
        }
    }

    pub async fn replenish_stock(
        &self,
        purchase_id: &str,
        items: Vec<ReplenishItemPayload>,
    ) -> Result<ReplenishStockResponse, AppError> {
        // 1. Try gRPC first
        match self.try_replenish_grpc(purchase_id, items.clone()).await {
            Ok(resp) => {
                info!(purchase_id = %purchase_id, "Stock replenished via gRPC successfully");
                return Ok(resp);
            }
            Err(e) => {
                warn!(error = %e, "Inventory gRPC replenish failed, falling back to HTTP REST");
            }
        }

        // 2. Fallback to HTTP REST with HMAC signature
        self.try_replenish_http(purchase_id, items).await
    }

    async fn try_replenish_grpc(
        &self,
        purchase_id: &str,
        items: Vec<ReplenishItemPayload>,
    ) -> Result<ReplenishStockResponse, AppError> {
        let channel = Channel::from_shared(self.grpc_url.clone())
            .map_err(|e| AppError::internal(&format!("Invalid gRPC URI: {}", e)))?
            .connect()
            .await
            .map_err(|e| {
                AppError::internal(&format!("Failed to connect to Inventory gRPC: {}", e))
            })?;

        let mut client = InventoryServiceClient::new(channel);
        let request = tonic::Request::new(ReplenishStockRequest {
            purchase_id: purchase_id.to_string(),
            items,
        });

        let response = client
            .replenish_stock(request)
            .await
            .map_err(|e| AppError::internal(&format!("Inventory gRPC error: {}", e)))?;

        Ok(response.into_inner())
    }

    async fn try_replenish_http(
        &self,
        purchase_id: &str,
        items: Vec<ReplenishItemPayload>,
    ) -> Result<ReplenishStockResponse, AppError> {
        #[derive(serde::Serialize)]
        struct ItemDto {
            ingredient_id: String,
            quantity: f64,
            unit_cost: f64,
        }

        #[derive(serde::Serialize)]
        struct PayloadDto {
            purchase_id: String,
            items: Vec<ItemDto>,
        }

        let body_dto = PayloadDto {
            purchase_id: purchase_id.to_string(),
            items: items
                .into_iter()
                .map(|i| ItemDto {
                    ingredient_id: i.ingredient_id,
                    quantity: i.quantity,
                    unit_cost: i.unit_cost,
                })
                .collect(),
        };

        let body_str = serde_json::to_string(&body_dto).map_err(|e| {
            AppError::internal(&format!("Failed to serialize body: {}", e))
        })?;

        let timestamp = chrono::Utc::now().to_rfc3339();
        let message = format!("{}{}", timestamp, body_str);
        let signature = generate_hmac(&self.app_secret, &message);

        let url = format!("{}/api/internal/stock/replenish", self.http_url);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        let resp = client
            .post(&url)
            .header("X-Signature", signature)
            .header("X-Timestamp", timestamp)
            .header("Content-Type", "application/json")
            .body(body_str)
            .send()
            .await
            .map_err(|e| AppError::internal(&format!("HTTP request failed: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_body = resp.text().await.unwrap_or_default();
            return Err(AppError::internal(&format!(
                "Inventory HTTP replenish failed ({}): {}",
                status, err_body
            )));
        }

        Ok(ReplenishStockResponse {
            success: true,
            message: format!("Stock replenished via HTTP REST for PO {}", purchase_id),
        })
    }
}
