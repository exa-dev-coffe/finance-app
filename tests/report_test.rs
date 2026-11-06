use chrono::Utc;
use rust_decimal::Decimal;
use finance_app::modules::report::model::FinancialSummaryReport;

#[test]
fn test_financial_summary_report() {
    let raw_material = Decimal::from(4500000);
    let operational = Decimal::from(1500000);
    let total = raw_material + operational;

    let report = FinancialSummaryReport {
        from: "2026-01-01".to_string(),
        to: "2026-12-31".to_string(),
        total_revenue: Decimal::from(10000000),
        cogs: raw_material,
        gross_profit: Decimal::from(5500000),
        gross_margin_percent: Decimal::from(55),
        total_raw_material_expense: raw_material,
        total_operational_expense: operational,
        total_spoilage_loss: Decimal::ZERO,
        total_expenses: total,
        net_profit: Decimal::from(4000000),
        net_margin_percent: Decimal::from(40),
        generated_at: Utc::now(),
    };

    assert_eq!(report.total_raw_material_expense, Decimal::from(4500000));
    assert_eq!(report.total_operational_expense, Decimal::from(1500000));
    assert_eq!(report.total_expenses, Decimal::from(6000000));
}

