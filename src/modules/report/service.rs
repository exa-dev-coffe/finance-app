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

        for (cat, amount) in rows {
            if cat == "RAW_MATERIAL" {
                raw_material += amount;
            } else {
                operational += amount;
            }
        }

        let total = raw_material + operational;

        Ok(FinancialSummaryReport {
            total_raw_material_expense: raw_material,
            total_operational_expense: operational,
            total_expenses: total,
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
