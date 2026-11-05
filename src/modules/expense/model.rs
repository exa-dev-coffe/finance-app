use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Expense {
    pub id: Uuid,
    pub expense_number: String,
    pub category: String,
    pub title: String,
    pub amount: Decimal,
    pub expense_date: DateTime<Utc>,
    pub payment_method: String,
    pub purchase_id: Option<Uuid>,
    pub receipt_url: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub created_by: String,
    pub approved_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateExpenseDto {
    pub category: String,
    pub title: String,
    pub amount: Decimal,
    pub payment_method: Option<String>,
    pub purchase_id: Option<Uuid>,
    pub receipt_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExpenseDto {
    pub category: Option<String>,
    pub title: Option<String>,
    pub amount: Option<Decimal>,
    pub payment_method: Option<String>,
    pub receipt_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExpenseQuery {
    pub category: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ExpenseCategorySummary {
    pub category: String,
    pub total_amount: Decimal,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct ExpenseSummaryResponse {
    pub total_expense: Decimal,
    pub categories: Vec<ExpenseCategorySummary>,
}

