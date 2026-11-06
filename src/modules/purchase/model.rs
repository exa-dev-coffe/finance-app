use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Purchase {
    pub id: Uuid,
    pub purchase_number: String,
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    pub purchase_date: DateTime<Utc>,
    pub status: String,
    pub payment_status: String,
    pub payment_method: String,
    pub total_amount: Decimal,
    pub paid_amount: Decimal,
    pub invoice_number: Option<String>,
    pub notes: Option<String>,
    pub created_by: String,
    pub received_at: Option<DateTime<Utc>>,
    pub received_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PurchaseItem {
    pub id: Uuid,
    pub purchase_id: Uuid,
    pub ingredient_id: Uuid,
    pub ingredient_name: String,
    pub unit: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub subtotal: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseDetailResponse {
    #[serde(flatten)]
    pub purchase: Purchase,
    pub items: Vec<PurchaseItem>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseItemDto {
    pub ingredient_id: Uuid,
    pub ingredient_name: String,
    pub unit: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseDto {
    pub supplier_id: Uuid,
    pub payment_method: Option<String>,
    pub invoice_number: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<CreatePurchaseItemDto>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReceiveItemPayload {
    pub ingredient_id: Uuid,
    pub quantity: Decimal,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ReceivePurchaseDto {
    pub items: Option<Vec<ReceiveItemPayload>>,
}

#[derive(Debug, Deserialize)]
pub struct PurchaseQuery {
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

