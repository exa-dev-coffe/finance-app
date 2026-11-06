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
    pub from: String,
    pub to: String,
    pub total_revenue: Decimal,
    pub cogs: Decimal,
    pub gross_profit: Decimal,
    pub gross_margin_percent: Decimal,
    pub total_raw_material_expense: Decimal,
    pub total_operational_expense: Decimal,
    #[serde(default)]
    pub total_spoilage_loss: Decimal,
    pub total_expenses: Decimal,
    pub net_profit: Decimal,
    pub net_margin_percent: Decimal,
    pub generated_at: DateTime<Utc>,
}

