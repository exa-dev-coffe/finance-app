use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::pkg::{
    auth::jwt::{AdminUser, StaffUser},
    errors::AppError,
    response::BaseResponse,
};
use super::model::{CreatePurchaseDto, PurchaseQuery};
use super::service::PurchaseService;

pub fn purchase_router(service: Arc<PurchaseService>) -> Router {
    Router::new()
        .route("/", get(list_purchases).post(create_purchase))
        .route("/:id", get(get_purchase))
        .route("/:id/receive", axum::routing::post(receive_purchase))
        .route("/:id/pay", axum::routing::post(pay_purchase))
        .route("/:id/cancel", axum::routing::post(cancel_purchase))
        .with_state(service)
}

async fn list_purchases(
    _staff: StaffUser,
    State(service): State<Arc<PurchaseService>>,
    Query(query): Query<PurchaseQuery>,
) -> Result<impl IntoResponse, AppError> {
    let purchases = service.list(query).await?;
    Ok(BaseResponse::success(purchases, "Purchases retrieved successfully"))
}

async fn create_purchase(
    staff: StaffUser,
    State(service): State<Arc<PurchaseService>>,
    Json(dto): Json<CreatePurchaseDto>,
) -> Result<impl IntoResponse, AppError> {
    let purchase = service.create(dto, &staff.email).await?;
    Ok(BaseResponse::created(purchase, "Purchase order created successfully"))
}

async fn get_purchase(
    _staff: StaffUser,
    State(service): State<Arc<PurchaseService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let detail = service.get_by_id(id).await?;
    Ok(BaseResponse::success(detail, "Purchase order details retrieved"))
}

async fn receive_purchase(
    staff: StaffUser,
    State(service): State<Arc<PurchaseService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let detail = service.receive_purchase(id, &staff.email).await?;
    Ok(BaseResponse::success(detail, "Purchase received and inventory replenished successfully"))
}

async fn pay_purchase(
    staff: StaffUser,
    State(service): State<Arc<PurchaseService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let detail = service.pay_purchase(id, &staff.email).await?;
    Ok(BaseResponse::success(detail, "Purchase marked as paid and expense recorded successfully"))
}

async fn cancel_purchase(
    _admin: AdminUser,
    State(service): State<Arc<PurchaseService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let detail = service.cancel_purchase(id).await?;
    Ok(BaseResponse::success(detail, "Purchase order cancelled successfully"))
}

