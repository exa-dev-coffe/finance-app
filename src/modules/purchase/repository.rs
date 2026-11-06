use rust_decimal::Decimal;
use sqlx::{PgPool, Result};
use uuid::Uuid;

use super::model::{
    CreatePurchaseDto, Purchase, PurchaseDetailResponse, PurchaseItem, PurchaseQuery,
};

#[derive(Clone)]
pub struct PurchaseRepository {
    pool: PgPool,
}

impl PurchaseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_with_items(
        &self,
        purchase_number: &str,
        dto: CreatePurchaseDto,
        created_by: &str,
    ) -> Result<PurchaseDetailResponse> {
        let mut tx = self.pool.begin().await?;

        let total_amount: Decimal = dto
            .items
            .iter()
            .map(|i| i.quantity * i.unit_price)
            .sum();

        let purchase: Purchase = sqlx::query_as(
            "INSERT INTO purchases (purchase_number, supplier_id, payment_method, total_amount, invoice_number, notes, created_by, status, payment_status)
             VALUES ($1, $2, COALESCE($3, 'BANK_TRANSFER'), $4, $5, $6, $7, 'ORDERED', 'UNPAID')
             RETURNING id, purchase_number, supplier_id, NULL as supplier_name, purchase_date, status, payment_status, payment_method, total_amount, paid_amount, invoice_number, notes, created_by, received_at, received_by, created_at, updated_at",
        )
        .bind(purchase_number)
        .bind(dto.supplier_id)
        .bind(dto.payment_method)
        .bind(total_amount)
        .bind(dto.invoice_number)
        .bind(dto.notes)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        let mut items = Vec::new();
        for it in dto.items {
            let subtotal = it.quantity * it.unit_price;
            let item: PurchaseItem = sqlx::query_as(
                "INSERT INTO purchase_items (purchase_id, ingredient_id, ingredient_name, unit, quantity, unit_price, subtotal)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)
                 RETURNING id, purchase_id, ingredient_id, ingredient_name, unit, quantity, unit_price, subtotal, created_at",
            )
            .bind(purchase.id)
            .bind(it.ingredient_id)
            .bind(it.ingredient_name)
            .bind(it.unit)
            .bind(it.quantity)
            .bind(it.unit_price)
            .bind(subtotal)
            .fetch_one(&mut *tx)
            .await?;

            items.push(item);
        }

        tx.commit().await?;

        Ok(PurchaseDetailResponse { purchase, items })
    }

    pub async fn list(&self, query: PurchaseQuery) -> Result<Vec<Purchase>> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        sqlx::query_as::<_, Purchase>(
            "SELECT p.id, p.purchase_number, p.supplier_id, s.name as supplier_name, p.purchase_date, p.status, p.payment_status, p.payment_method, p.total_amount, p.paid_amount, p.invoice_number, p.notes, p.created_by, p.received_at, p.received_by, p.created_at, p.updated_at
             FROM purchases p
             JOIN suppliers s ON p.supplier_id = s.id
             WHERE ($3::TEXT IS NULL OR p.status = $3)
               AND ($4::TEXT IS NULL OR p.payment_status = $4)
               AND ($5::UUID IS NULL OR p.supplier_id = $5)
             ORDER BY p.purchase_date DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .bind(query.status)
        .bind(query.payment_status)
        .bind(query.supplier_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<PurchaseDetailResponse>> {
        let purchase: Option<Purchase> = sqlx::query_as(
            "SELECT p.id, p.purchase_number, p.supplier_id, s.name as supplier_name, p.purchase_date, p.status, p.payment_status, p.payment_method, p.total_amount, p.paid_amount, p.invoice_number, p.notes, p.created_by, p.received_at, p.received_by, p.created_at, p.updated_at
             FROM purchases p
             JOIN suppliers s ON p.supplier_id = s.id
             WHERE p.id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let purchase = match purchase {
            Some(p) => p,
            None => return Ok(None),
        };

        let items: Vec<PurchaseItem> = sqlx::query_as(
            "SELECT id, purchase_id, ingredient_id, ingredient_name, unit, quantity, unit_price, subtotal, created_at
             FROM purchase_items
             WHERE purchase_id = $1
             ORDER BY ingredient_name ASC",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(PurchaseDetailResponse { purchase, items }))
    }

    pub async fn update_status(&self, id: Uuid, status: &str, received_by: &str) -> Result<Option<Purchase>> {
        sqlx::query_as::<_, Purchase>(
            "UPDATE purchases
             SET status = $2, received_at = NOW(), received_by = $3, updated_at = NOW()
             WHERE id = $1 AND status != 'CANCELLED'
             RETURNING id, purchase_number, supplier_id, NULL as supplier_name, purchase_date, status, payment_status, payment_method, total_amount, paid_amount, invoice_number, notes, created_by, received_at, received_by, created_at, updated_at",
        )
        .bind(id)
        .bind(status)
        .bind(received_by)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn mark_as_received(&self, id: Uuid, received_by: &str) -> Result<Option<Purchase>> {
        self.update_status(id, "RECEIVED", received_by).await
    }

    pub async fn mark_as_paid(&self, id: Uuid) -> Result<Option<Purchase>> {
        sqlx::query_as::<_, Purchase>(
            "UPDATE purchases
             SET payment_status = 'PAID', paid_amount = total_amount, updated_at = NOW()
             WHERE id = $1 AND payment_status != 'PAID'
             RETURNING id, purchase_number, supplier_id, NULL as supplier_name, purchase_date, status, payment_status, payment_method, total_amount, paid_amount, invoice_number, notes, created_by, received_at, received_by, created_at, updated_at",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn cancel(&self, id: Uuid) -> Result<Option<Purchase>> {
        sqlx::query_as::<_, Purchase>(
            "UPDATE purchases
             SET status = 'CANCELLED', updated_at = NOW()
             WHERE id = $1 AND status != 'RECEIVED'
             RETURNING id, purchase_number, supplier_id, NULL as supplier_name, purchase_date, status, payment_status, payment_method, total_amount, paid_amount, invoice_number, notes, created_by, received_at, received_by, created_at, updated_at",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }
}

