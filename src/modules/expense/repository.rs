use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Result};
use uuid::Uuid;

use super::model::{
    CreateExpenseDto, Expense, ExpenseCategorySummary, ExpenseQuery, ExpenseSummaryResponse,
    UpdateExpenseDto,
};

#[derive(Clone)]
pub struct ExpenseRepository {
    pool: PgPool,
}

impl ExpenseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        expense_number: &str,
        dto: CreateExpenseDto,
        created_by: &str,
    ) -> Result<Expense> {
        sqlx::query_as::<_, Expense>(
            "INSERT INTO expenses (expense_number, category, title, amount, payment_method, purchase_id, receipt_url, notes, created_by, status)
             VALUES ($1, $2, $3, $4, COALESCE($5, 'CASH'), $6, $7, $8, $9, 'PAID')
             RETURNING id, expense_number, category, title, amount, expense_date, payment_method, purchase_id, receipt_url, notes, status, created_by, approved_by, created_at, updated_at",
        )
        .bind(expense_number)
        .bind(dto.category)
        .bind(dto.title)
        .bind(dto.amount)
        .bind(dto.payment_method)
        .bind(dto.purchase_id)
        .bind(dto.receipt_url)
        .bind(dto.notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list(&self, query: ExpenseQuery) -> Result<Vec<Expense>> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        sqlx::query_as::<_, Expense>(
            "SELECT id, expense_number, category, title, amount, expense_date, payment_method, purchase_id, receipt_url, notes, status, created_by, approved_by, created_at, updated_at
             FROM expenses
             WHERE ($3::TEXT IS NULL OR category = $3)
               AND ($4::TEXT IS NULL OR status = $4)
               AND ($5::TIMESTAMPTZ IS NULL OR expense_date >= $5)
               AND ($6::TIMESTAMPTZ IS NULL OR expense_date <= $6)
             ORDER BY expense_date DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .bind(query.category)
        .bind(query.status)
        .bind(query.start_date)
        .bind(query.end_date)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Expense>> {
        sqlx::query_as::<_, Expense>(
            "SELECT id, expense_number, category, title, amount, expense_date, payment_method, purchase_id, receipt_url, notes, status, created_by, approved_by, created_at, updated_at
             FROM expenses
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update(&self, id: Uuid, dto: UpdateExpenseDto) -> Result<Option<Expense>> {
        sqlx::query_as::<_, Expense>(
            "UPDATE expenses
             SET category = COALESCE($2, category),
                 title = COALESCE($3, title),
                 amount = COALESCE($4, amount),
                 payment_method = COALESCE($5, payment_method),
                 receipt_url = COALESCE($6, receipt_url),
                 notes = COALESCE($7, notes),
                 updated_at = NOW()
             WHERE id = $1
             RETURNING id, expense_number, category, title, amount, expense_date, payment_method, purchase_id, receipt_url, notes, status, created_by, approved_by, created_at, updated_at",
        )
        .bind(id)
        .bind(dto.category)
        .bind(dto.title)
        .bind(dto.amount)
        .bind(dto.payment_method)
        .bind(dto.receipt_url)
        .bind(dto.notes)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let rows = sqlx::query("DELETE FROM expenses WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        Ok(rows > 0)
    }

    pub async fn get_summary(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<ExpenseSummaryResponse> {
        let categories = sqlx::query_as::<_, ExpenseCategorySummary>(
            "SELECT category, COALESCE(SUM(amount), 0) as total_amount, COUNT(*) as count
             FROM expenses
             WHERE ($1::TIMESTAMPTZ IS NULL OR expense_date >= $1)
               AND ($2::TIMESTAMPTZ IS NULL OR expense_date <= $2)
             GROUP BY category
             ORDER BY total_amount DESC",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        let total_expense: Decimal = categories.iter().map(|c| c.total_amount).sum();

        Ok(ExpenseSummaryResponse {
            total_expense,
            categories,
        })
    }
}
