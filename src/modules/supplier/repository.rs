use sqlx::{PgPool, Result};
use uuid::Uuid;

use super::model::{CreateSupplierDto, Supplier, SupplierQuery, UpdateSupplierDto};

#[derive(Clone)]
pub struct SupplierRepository {
    pool: PgPool,
}

impl SupplierRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, dto: CreateSupplierDto) -> Result<Supplier> {
        sqlx::query_as::<_, Supplier>(
            "INSERT INTO suppliers (code, name, contact_person, phone, email, address, payment_terms)
             VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, 'NET_30'))
             RETURNING id, code, name, contact_person, phone, email, address, payment_terms, is_active, created_at, updated_at",
        )
        .bind(dto.code)
        .bind(dto.name)
        .bind(dto.contact_person)
        .bind(dto.phone)
        .bind(dto.email)
        .bind(dto.address)
        .bind(dto.payment_terms)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list(&self, query: SupplierQuery) -> Result<Vec<Supplier>> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        sqlx::query_as::<_, Supplier>(
            "SELECT id, code, name, contact_person, phone, email, address, payment_terms, is_active, created_at, updated_at
             FROM suppliers
             WHERE ($3::BOOLEAN IS NULL OR is_active = $3)
               AND ($4::TEXT IS NULL OR name ILIKE '%' || $4 || '%' OR code ILIKE '%' || $4 || '%')
             ORDER BY name ASC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .bind(query.is_active)
        .bind(query.search)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Supplier>> {
        sqlx::query_as::<_, Supplier>(
            "SELECT id, code, name, contact_person, phone, email, address, payment_terms, is_active, created_at, updated_at
             FROM suppliers
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update(&self, id: Uuid, dto: UpdateSupplierDto) -> Result<Option<Supplier>> {
        sqlx::query_as::<_, Supplier>(
            "UPDATE suppliers
             SET name = COALESCE($2, name),
                 contact_person = COALESCE($3, contact_person),
                 phone = COALESCE($4, phone),
                 email = COALESCE($5, email),
                 address = COALESCE($6, address),
                 payment_terms = COALESCE($7, payment_terms),
                 is_active = COALESCE($8, is_active),
                 updated_at = NOW()
             WHERE id = $1
             RETURNING id, code, name, contact_person, phone, email, address, payment_terms, is_active, created_at, updated_at",
        )
        .bind(id)
        .bind(dto.name)
        .bind(dto.contact_person)
        .bind(dto.phone)
        .bind(dto.email)
        .bind(dto.address)
        .bind(dto.payment_terms)
        .bind(dto.is_active)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let rows = sqlx::query("DELETE FROM suppliers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        Ok(rows > 0)
    }
}
