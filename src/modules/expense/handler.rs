use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::pkg::{
    auth::jwt::{AdminUser, StaffUser},
    errors::AppError,
    response::BaseResponse,
};
use super::model::{CreateExpenseDto, ExpenseQuery, UpdateExpenseDto};
use super::service::ExpenseService;

#[derive(Debug, Deserialize)]
pub struct SummaryQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

pub fn expense_router(service: Arc<ExpenseService>) -> Router {
    Router::new()
        .route("/", get(list_expenses).post(create_expense))
        .route("/summary", get(get_expense_summary))
        .route("/:id", get(get_expense).put(update_expense).delete(delete_expense))
        .with_state(service)
}

async fn list_expenses(
    _staff: StaffUser,
    State(service): State<Arc<ExpenseService>>,
    Query(query): Query<ExpenseQuery>,
) -> Result<impl IntoResponse, AppError> {
    let expenses = service.list(query).await?;
    Ok(BaseResponse::success(expenses, "Expenses retrieved successfully"))
}

async fn get_expense_summary(
    _staff: StaffUser,
    State(service): State<Arc<ExpenseService>>,
    Query(query): Query<SummaryQuery>,
) -> Result<impl IntoResponse, AppError> {
    let summary = service.get_summary(query.start_date, query.end_date).await?;
    Ok(BaseResponse::success(summary, "Expense summary calculated successfully"))
}

async fn create_expense(
    staff: StaffUser,
    State(service): State<Arc<ExpenseService>>,
    Json(dto): Json<CreateExpenseDto>,
) -> Result<impl IntoResponse, AppError> {
    let expense = service.create(dto, &staff.email).await?;
    Ok(BaseResponse::created(expense, "Expense recorded successfully"))
}

async fn get_expense(
    _staff: StaffUser,
    State(service): State<Arc<ExpenseService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let expense = service.get_by_id(id).await?;
    Ok(BaseResponse::success(expense, "Expense details retrieved"))
}

async fn update_expense(
    _admin: AdminUser,
    State(service): State<Arc<ExpenseService>>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateExpenseDto>,
) -> Result<impl IntoResponse, AppError> {
    let expense = service.update(id, dto).await?;
    Ok(BaseResponse::success(expense, "Expense updated successfully"))
}

async fn delete_expense(
    _admin: AdminUser,
    State(service): State<Arc<ExpenseService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service.delete(id).await?;
    Ok(BaseResponse::ok_msg("Expense deleted successfully"))
}
