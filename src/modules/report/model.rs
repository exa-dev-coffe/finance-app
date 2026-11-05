use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct MonthlyExpenseReport {
    pub month: String,
    pub category: String,
    pub total: Decimal,
}

#[derive(Debug, Serialize)]
pub struct FinancialSummaryReport {
    pub total_raw_material_expense: Decimal,
    pub total_operational_expense: Decimal,
    pub total_expenses: Decimal,
    pub generated_at: DateTime<Utc>,
}

