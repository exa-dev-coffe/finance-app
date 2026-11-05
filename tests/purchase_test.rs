use rust_decimal::Decimal;
use uuid::Uuid;

use finance_app::modules::purchase::model::{CreatePurchaseDto, CreatePurchaseItemDto};

#[test]
fn test_create_purchase_dto_and_totals() {
    let supplier_id = Uuid::new_v4();
    let ing1 = Uuid::new_v4();
    let ing2 = Uuid::new_v4();

    let items = vec![
        CreatePurchaseItemDto {
            ingredient_id: ing1,
            ingredient_name: "Biji Kopi Arabika 1kg".to_string(),
            unit: "kg".to_string(),
            quantity: Decimal::from(10), // 10 kg
            unit_price: Decimal::from(120000), // Rp 120.000 / kg
        },
        CreatePurchaseItemDto {
            ingredient_id: ing2,
            ingredient_name: "Susu Fresh Milk 1L".to_string(),
            unit: "liter".to_string(),
            quantity: Decimal::from(20), // 20 L
            unit_price: Decimal::from(18000), // Rp 18.000 / L
        },
    ];

    let total: Decimal = items.iter().map(|i| i.quantity * i.unit_price).sum();
    // 10 * 120.000 = 1.200.000, 20 * 18.000 = 360.000 -> Total = 1.560.000
    assert_eq!(total, Decimal::from(1560000));

    let dto = CreatePurchaseDto {
        supplier_id,
        payment_method: Some("BANK_TRANSFER".to_string()),
        invoice_number: Some("INV-2026-001".to_string()),
        notes: Some("Restock mingguan".to_string()),
        items,
    };

    assert_eq!(dto.items.len(), 2);
    assert_eq!(dto.payment_method.unwrap(), "BANK_TRANSFER");
}
