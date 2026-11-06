use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;

use super::model::{FinancialSummaryReport, MonthlyExpenseReport};
use crate::pkg::errors::AppError;

#[derive(Clone)]
pub struct ReportService {
    pool: PgPool,
}

impl ReportService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_financial_summary(&self) -> Result<FinancialSummaryReport, AppError> {
        let rows: Vec<(String, Decimal)> = sqlx::query_as(
            "SELECT category, COALESCE(SUM(amount), 0)
             FROM expenses
             GROUP BY category",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        let mut raw_material = Decimal::ZERO;
        let mut operational = Decimal::ZERO;
        let mut spoilage = Decimal::ZERO;

        for (cat, amount) in rows {
            if cat == "RAW_MATERIAL" {
                raw_material += amount;
            } else if cat == "SPOILAGE" || cat == "WASTE_LOSS" {
                spoilage += amount;
                operational += amount;
            } else {
                operational += amount;
            }
        }

        let total = raw_material + operational;
        let cogs = raw_material;
        let total_revenue = Decimal::ZERO;
        let gross_profit = total_revenue - cogs;
        let net_profit = gross_profit - operational;
        let gross_margin_percent = Decimal::ZERO;
        let net_margin_percent = Decimal::ZERO;

        Ok(FinancialSummaryReport {
            from: "".to_string(),
            to: "".to_string(),
            total_revenue,
            cogs,
            gross_profit,
            gross_margin_percent,
            total_raw_material_expense: raw_material,
            total_operational_expense: operational,
            total_spoilage_loss: spoilage,
            total_expenses: total,
            net_profit,
            net_margin_percent,
            generated_at: Utc::now(),
        })
    }

    pub async fn get_monthly_breakdown(&self) -> Result<Vec<MonthlyExpenseReport>, AppError> {
        sqlx::query_as::<_, MonthlyExpenseReport>(
            "SELECT TO_CHAR(expense_date, 'YYYY-MM') as month, category, COALESCE(SUM(amount), 0) as total
             FROM expenses
             GROUP BY TO_CHAR(expense_date, 'YYYY-MM'), category
             ORDER BY month DESC, category ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)
    }
}
