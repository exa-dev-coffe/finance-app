use chrono::{DateTime, Utc};
use rand::Rng;
use std::sync::Arc;
use uuid::Uuid;

use super::model::{
    CreateExpenseDto, Expense, ExpenseQuery, ExpenseSummaryResponse, UpdateExpenseDto,
};
use super::repository::ExpenseRepository;
use crate::pkg::errors::AppError;

#[derive(Clone)]
pub struct ExpenseService {
    repo: Arc<ExpenseRepository>,
}

impl ExpenseService {
    pub fn new(repo: Arc<ExpenseRepository>) -> Self {
        Self { repo }
    }

    fn generate_expense_number() -> String {
        let now = Utc::now();
        let rand_suffix: u32 = rand::thread_rng().gen_range(1000..9999);
        format!("EXP-{}-{:04}", now.format("%Y%m%d"), rand_suffix)
    }

    pub async fn create(&self, dto: CreateExpenseDto, created_by: &str) -> Result<Expense, AppError> {
        let expense_number = Self::generate_expense_number();
        self.repo
            .create(&expense_number, dto, created_by)
            .await
            .map_err(AppError::Database)
    }

    pub async fn list(&self, query: ExpenseQuery) -> Result<Vec<Expense>, AppError> {
        self.repo.list(query).await.map_err(AppError::Database)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Expense, AppError> {
        self.repo
            .get_by_id(id)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::not_found("Expense record not found", "EXPENSE_NOT_FOUND"))
    }

    pub async fn update(&self, id: Uuid, dto: UpdateExpenseDto) -> Result<Expense, AppError> {
        self.repo
            .update(id, dto)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::not_found("Expense record not found", "EXPENSE_NOT_FOUND"))
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let deleted = self.repo.delete(id).await.map_err(AppError::Database)?;
        if !deleted {
            return Err(AppError::not_found("Expense record not found", "EXPENSE_NOT_FOUND"));
        }
        Ok(())
    }

    pub async fn get_summary(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<ExpenseSummaryResponse, AppError> {
        self.repo
            .get_summary(start_date, end_date)
            .await
            .map_err(AppError::Database)
    }
}
