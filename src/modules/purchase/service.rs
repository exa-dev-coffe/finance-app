use chrono::Utc;
use rand::Rng;
use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;
use uuid::Uuid;

use super::model::{
    CreatePurchaseDto, Purchase, PurchaseDetailResponse, PurchaseQuery, ReceivePurchaseDto,
};
use super::repository::PurchaseRepository;
use crate::modules::expense::model::CreateExpenseDto;
use crate::modules::expense::service::ExpenseService;
use crate::modules::grpc::{InventoryClient, ReplenishItemPayload};
use crate::pkg::errors::AppError;

#[derive(Clone)]
pub struct PurchaseService {
    repo: Arc<PurchaseRepository>,
    inventory_client: Arc<InventoryClient>,
    expense_service: Arc<ExpenseService>,
}

impl PurchaseService {
    pub fn new(
        repo: Arc<PurchaseRepository>,
        inventory_client: Arc<InventoryClient>,
        expense_service: Arc<ExpenseService>,
    ) -> Self {
        Self {
            repo,
            inventory_client,
            expense_service,
        }
    }

    fn generate_po_number() -> String {
        let now = Utc::now();
        let rand_suffix: u32 = rand::thread_rng().gen_range(1000..9999);
        format!("PO-{}-{:04}", now.format("%Y%m%d"), rand_suffix)
    }

    pub async fn create(
        &self,
        dto: CreatePurchaseDto,
        created_by: &str,
    ) -> Result<PurchaseDetailResponse, AppError> {
        if dto.items.is_empty() {
            return Err(AppError::bad_request("Purchase must contain at least one item", "EMPTY_ITEMS"));
        }

        let po_number = Self::generate_po_number();
        self.repo
            .create_with_items(&po_number, dto, created_by)
            .await
            .map_err(AppError::Database)
    }

    pub async fn list(&self, query: PurchaseQuery) -> Result<Vec<Purchase>, AppError> {
        self.repo.list(query).await.map_err(AppError::Database)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<PurchaseDetailResponse, AppError> {
        self.repo
            .get_by_id(id)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::not_found("Purchase order not found", "PO_NOT_FOUND"))
    }

    pub async fn receive_purchase(
        &self,
        id: Uuid,
        received_by: &str,
        receive_dto: Option<ReceivePurchaseDto>,
    ) -> Result<PurchaseDetailResponse, AppError> {
        let detail = self.get_by_id(id).await?;
        if detail.purchase.status == "RECEIVED" {
            return Err(AppError::bad_request(
                "Purchase order has already been received",
                "ALREADY_RECEIVED",
            ));
        }
        if detail.purchase.status == "CANCELLED" {
            return Err(AppError::bad_request(
                "Cannot receive a cancelled purchase order",
                "PO_CANCELLED",
            ));
        }

        // Determine items and quantities to replenish (supports partial delivery)
        let mut is_partial = false;
        let replenish_items: Vec<ReplenishItemPayload> = if let Some(dto) = receive_dto {
            if let Some(custom_items) = dto.items {
                if !custom_items.is_empty() {
                    let mut res = Vec::new();
                    for po_item in &detail.items {
                        if let Some(matched) = custom_items.iter().find(|i| i.ingredient_id == po_item.ingredient_id) {
                            if matched.quantity < po_item.quantity {
                                is_partial = true;
                            }
                            res.push(ReplenishItemPayload {
                                ingredient_id: matched.ingredient_id.to_string(),
                                quantity: matched.quantity.to_f64().unwrap_or(0.0),
                                unit_cost: po_item.unit_price.to_f64().unwrap_or(0.0),
                            });
                        }
                    }
                    res
                } else {
                    detail
                        .items
                        .iter()
                        .map(|it| ReplenishItemPayload {
                            ingredient_id: it.ingredient_id.to_string(),
                            quantity: it.quantity.to_f64().unwrap_or(0.0),
                            unit_cost: it.unit_price.to_f64().unwrap_or(0.0),
                        })
                        .collect()
                }
            } else {
                detail
                    .items
                    .iter()
                    .map(|it| ReplenishItemPayload {
                        ingredient_id: it.ingredient_id.to_string(),
                        quantity: it.quantity.to_f64().unwrap_or(0.0),
                        unit_cost: it.unit_price.to_f64().unwrap_or(0.0),
                    })
                    .collect()
            }
        } else {
            detail
                .items
                .iter()
                .map(|it| ReplenishItemPayload {
                    ingredient_id: it.ingredient_id.to_string(),
                    quantity: it.quantity.to_f64().unwrap_or(0.0),
                    unit_cost: it.unit_price.to_f64().unwrap_or(0.0),
                })
                .collect()
        };

        // 1. Replenish inventory via gRPC (with REST fallback)
        self.inventory_client
            .replenish_stock(&detail.purchase.purchase_number, replenish_items)
            .await?;

        // 2. Mark as RECEIVED or PARTIALLY_RECEIVED in database
        let new_status = if is_partial { "PARTIALLY_RECEIVED" } else { "RECEIVED" };
        self.repo
            .update_status(id, new_status, received_by)
            .await
            .map_err(AppError::Database)?;

        self.get_by_id(id).await
    }

    pub async fn pay_purchase(&self, id: Uuid, paid_by: &str) -> Result<PurchaseDetailResponse, AppError> {
        let detail = self.get_by_id(id).await?;
        if detail.purchase.payment_status == "PAID" {
            return Err(AppError::bad_request(
                "Purchase order is already paid",
                "ALREADY_PAID",
            ));
        }

        // 1. Mark as PAID in database
        self.repo
            .mark_as_paid(id)
            .await
            .map_err(AppError::Database)?;

        // 2. Automatically record Expense in RAW_MATERIAL category
        let expense_dto = CreateExpenseDto {
            category: "RAW_MATERIAL".to_string(),
            title: format!(
                "PO Payment - {} ({})",
                detail.purchase.purchase_number,
                detail.purchase.supplier_name.as_deref().unwrap_or("Supplier")
            ),
            amount: detail.purchase.total_amount,
            payment_method: Some(detail.purchase.payment_method.clone()),
            purchase_id: Some(id),
            receipt_url: None,
            notes: Some(format!(
                "Auto-generated expense for invoice {}",
                detail.purchase.invoice_number.as_deref().unwrap_or("-")
            )),
        };

        let _ = self.expense_service.create(expense_dto, paid_by).await?;

        self.get_by_id(id).await
    }

    pub async fn cancel_purchase(&self, id: Uuid) -> Result<PurchaseDetailResponse, AppError> {
        let updated = self.repo.cancel(id).await.map_err(AppError::Database)?;
        if updated.is_none() {
            return Err(AppError::bad_request(
                "Cannot cancel a received purchase order",
                "CANNOT_CANCEL",
            ));
        }
        self.get_by_id(id).await
    }
}

