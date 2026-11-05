use std::sync::Arc;
use uuid::Uuid;

use super::model::{CreateSupplierDto, Supplier, SupplierQuery, UpdateSupplierDto};
use super::repository::SupplierRepository;
use crate::pkg::errors::AppError;

#[derive(Clone)]
pub struct SupplierService {
    repo: Arc<SupplierRepository>,
}

impl SupplierService {
    pub fn new(repo: Arc<SupplierRepository>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, dto: CreateSupplierDto) -> Result<Supplier, AppError> {
        self.repo.create(dto).await.map_err(AppError::Database)
    }

    pub async fn list(&self, query: SupplierQuery) -> Result<Vec<Supplier>, AppError> {
        self.repo.list(query).await.map_err(AppError::Database)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Supplier, AppError> {
        self.repo
            .get_by_id(id)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::not_found("Supplier not found", "SUPPLIER_NOT_FOUND"))
    }

    pub async fn update(&self, id: Uuid, dto: UpdateSupplierDto) -> Result<Supplier, AppError> {
        self.repo
            .update(id, dto)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::not_found("Supplier not found", "SUPPLIER_NOT_FOUND"))
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let deleted = self.repo.delete(id).await.map_err(AppError::Database)?;
        if !deleted {
            return Err(AppError::not_found("Supplier not found", "SUPPLIER_NOT_FOUND"));
        }
        Ok(())
    }
}

