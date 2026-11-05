use rust_decimal::Decimal;
use finance_app::modules::expense::model::{
    CreateExpenseDto, ExpenseCategorySummary, ExpenseSummaryResponse,
};

#[test]
fn test_create_expense_dto() {
    let dto = CreateExpenseDto {
        category: "OPERATIONAL".to_string(),
        title: "Tagihan Listrik PLN Bulan Ini".to_string(),
        amount: Decimal::from(850000),
        payment_method: Some("BANK_TRANSFER".to_string()),
        purchase_id: None,
        receipt_url: Some("https://storage.coffe.dev/receipts/pln.pdf".to_string()),
        notes: Some("Pemakaian daya 1300VA".to_string()),
    };

    assert_eq!(dto.category, "OPERATIONAL");
    assert_eq!(dto.amount, Decimal::from(850000));
}

#[test]
fn test_expense_summary_calculation() {
    let cat1 = ExpenseCategorySummary {
        category: "RAW_MATERIAL".to_string(),
        total_amount: Decimal::from(3500000),
        count: 5,
    };
    let cat2 = ExpenseCategorySummary {
        category: "OPERATIONAL".to_string(),
        total_amount: Decimal::from(1200000),
        count: 3,
    };
    let cat3 = ExpenseCategorySummary {
        category: "SUPPLIES".to_string(),
        total_amount: Decimal::from(450000),
        count: 2,
    };

    let summary = ExpenseSummaryResponse {
        total_expense: cat1.total_amount + cat2.total_amount + cat3.total_amount,
        categories: vec![cat1, cat2, cat3],
    };

    assert_eq!(summary.total_expense, Decimal::from(5150000));
    assert_eq!(summary.categories.len(), 3);
}
