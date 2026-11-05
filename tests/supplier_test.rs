use finance_app::modules::supplier::model::{CreateSupplierDto, UpdateSupplierDto};

#[test]
fn test_create_supplier_dto() {
    let dto = CreateSupplierDto {
        code: "SUP-GAYO-01".to_string(),
        name: "PT Kopi Gayo Nusantara".to_string(),
        contact_person: Some("Budi Santoso".to_string()),
        phone: Some("08123456789".to_string()),
        email: Some("budi@gayocoffee.com".to_string()),
        address: Some("Takengon, Aceh Tengah".to_string()),
        payment_terms: Some("NET_30".to_string()),
    };

    assert_eq!(dto.code, "SUP-GAYO-01");
    assert_eq!(dto.name, "PT Kopi Gayo Nusantara");
    assert_eq!(dto.payment_terms.unwrap(), "NET_30");
}

#[test]
fn test_update_supplier_dto() {
    let dto = UpdateSupplierDto {
        name: Some("PT Kopi Gayo Mandiri".to_string()),
        contact_person: None,
        phone: Some("08198765432".to_string()),
        email: None,
        address: None,
        payment_terms: Some("NET_14".to_string()),
        is_active: Some(false),
    };

    assert_eq!(dto.name.unwrap(), "PT Kopi Gayo Mandiri");
    assert_eq!(dto.is_active, Some(false));
}
