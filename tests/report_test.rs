use chrono::Utc;
use rust_decimal::Decimal;
use finance_app::modules::report::model::FinancialSummaryReport;

#[test]
fn test_financial_summary_report() {
    let raw_material = Decimal::from(4500000);
    let operational = Decimal::from(1500000);
    let total = raw_material + operational;

    let report = FinancialSummaryReport {
        total_raw_material_expense: raw_material,
        total_operational_expense: operational,
        total_expenses: total,
        generated_at: Utc::now(),
    };

    assert_eq!(report.total_raw_material_expense, Decimal::from(4500000));
    assert_eq!(report.total_operational_expense, Decimal::from(1500000));
    assert_eq!(report.total_expenses, Decimal::from(6000000));
}

