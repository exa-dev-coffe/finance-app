use axum::{extract::State, response::IntoResponse, routing::get, Router};
use std::sync::Arc;

use crate::pkg::{auth::jwt::StaffUser, errors::AppError, response::BaseResponse};
use super::service::ReportService;

pub fn report_router(service: Arc<ReportService>) -> Router {
    Router::new()
        .route("/financial-summary", get(get_financial_summary))
        .route("/monthly-expenses", get(get_monthly_breakdown))
        .with_state(service)
}

async fn get_financial_summary(
    _staff: StaffUser,
    State(service): State<Arc<ReportService>>,
) -> Result<impl IntoResponse, AppError> {
    let report = service.get_financial_summary().await?;
    Ok(BaseResponse::success(report, "Financial summary calculated successfully"))
}

async fn get_monthly_breakdown(
    _staff: StaffUser,
    State(service): State<Arc<ReportService>>,
) -> Result<impl IntoResponse, AppError> {
    let report = service.get_monthly_breakdown().await?;
    Ok(BaseResponse::success(report, "Monthly expense breakdown retrieved successfully"))
}
