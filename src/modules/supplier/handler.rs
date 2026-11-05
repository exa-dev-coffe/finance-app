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
use super::model::{CreateSupplierDto, SupplierQuery, UpdateSupplierDto};
use super::service::SupplierService;

pub fn supplier_router(service: Arc<SupplierService>) -> Router {
    Router::new()
        .route("/", get(list_suppliers).post(create_supplier))
        .route("/:id", get(get_supplier).put(update_supplier).delete(delete_supplier))
        .with_state(service)
}

async fn list_suppliers(
    _staff: StaffUser,
    State(service): State<Arc<SupplierService>>,
    Query(query): Query<SupplierQuery>,
) -> Result<impl IntoResponse, AppError> {
    let suppliers = service.list(query).await?;
    Ok(BaseResponse::success(suppliers, "Suppliers retrieved successfully"))
}

async fn create_supplier(
    _admin: AdminUser,
    State(service): State<Arc<SupplierService>>,
    Json(dto): Json<CreateSupplierDto>,
) -> Result<impl IntoResponse, AppError> {
    let supplier = service.create(dto).await?;
    Ok(BaseResponse::created(supplier, "Supplier created successfully"))
}

async fn get_supplier(
    _staff: StaffUser,
    State(service): State<Arc<SupplierService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let supplier = service.get_by_id(id).await?;
    Ok(BaseResponse::success(supplier, "Supplier details retrieved"))
}

async fn update_supplier(
    _admin: AdminUser,
    State(service): State<Arc<SupplierService>>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateSupplierDto>,
) -> Result<impl IntoResponse, AppError> {
    let supplier = service.update(id, dto).await?;
    Ok(BaseResponse::success(supplier, "Supplier updated successfully"))
}

async fn delete_supplier(
    _admin: AdminUser,
    State(service): State<Arc<SupplierService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service.delete(id).await?;
    Ok(BaseResponse::ok_msg("Supplier deleted successfully"))
}

