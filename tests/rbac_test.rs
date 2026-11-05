use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, Request},
};
use finance_app::pkg::auth::{
    jwt::{
        generate_token_pair, generate_token_pair_with_role_id, AdminUser, OptionalAuthUser,
        StaffUser,
    },
    permission::{
        clear_test_role_permissions, set_test_role_permissions, PermissionAction,
        RequireFinanceView, RequirePurchaseCreate, RequirePurchaseView, RequireSupplierDelete,
    },
};
use std::collections::HashMap;

#[tokio::test]
async fn test_admin_user_extractor() {
    let secret = "test-secret-key-12345678901234567890";
    std::env::set_var("JWT_SECRET", secret);

    let admin_id = "1"; // Account-app numeric user id
    let admin_tokens =
        generate_token_pair(admin_id, "admin@example.com", "admin", secret, 15).unwrap();

    let user_id = "42";
    let user_tokens =
        generate_token_pair(user_id, "user@example.com", "customer", secret, 15).unwrap();

    // 1. Admin token should succeed
    let req = Request::builder()
        .header(
            AUTHORIZATION,
            format!("Bearer {}", admin_tokens.access_token),
        )
        .body(())
        .unwrap();
    let (mut parts, _) = req.into_parts();
    let admin = AdminUser::from_request_parts(&mut parts, &()).await;
    assert!(admin.is_ok());
    assert_eq!(admin.unwrap().role, "admin");

    // 2. Regular customer token should fail with forbidden
    let req_user = Request::builder()
        .header(
            AUTHORIZATION,
            format!("Bearer {}", user_tokens.access_token),
        )
        .body(())
        .unwrap();
    let (mut parts_user, _) = req_user.into_parts();
    let user_res = AdminUser::from_request_parts(&mut parts_user, &()).await;
    assert!(user_res.is_err());
}

#[tokio::test]
async fn test_staff_user_extractor() {
    let secret = "test-secret-key-12345678901234567890";
    std::env::set_var("JWT_SECRET", secret);

    let cashier_id = "2";
    let cashier_tokens =
        generate_token_pair(cashier_id, "cashier@example.com", "cashier", secret, 15).unwrap();

    let customer_id = "3";
    let customer_tokens =
        generate_token_pair(customer_id, "customer@example.com", "customer", secret, 15).unwrap();

    // 1. Cashier token should succeed for StaffUser
    let req = Request::builder()
        .header(
            AUTHORIZATION,
            format!("Bearer {}", cashier_tokens.access_token),
        )
        .body(())
        .unwrap();
    let (mut parts, _) = req.into_parts();
    let staff = StaffUser::from_request_parts(&mut parts, &()).await;
    assert!(staff.is_ok());
    assert_eq!(staff.unwrap().role, "cashier");

    // 2. Customer token should fail for StaffUser
    let req_cust = Request::builder()
        .header(
            AUTHORIZATION,
            format!("Bearer {}", customer_tokens.access_token),
        )
        .body(())
        .unwrap();
    let (mut parts_cust, _) = req_cust.into_parts();
    let cust_res = StaffUser::from_request_parts(&mut parts_cust, &()).await;
    assert!(cust_res.is_err());
}

#[tokio::test]
async fn test_optional_auth_user_extractor() {
    let secret = "test-secret-key-12345678901234567890";
    std::env::set_var("JWT_SECRET", secret);

    let user_id = "42";
    let tokens = generate_token_pair(user_id, "opt@example.com", "user", secret, 15).unwrap();

    // 1. Present valid token
    let req = Request::builder()
        .header(AUTHORIZATION, format!("Bearer {}", tokens.access_token))
        .body(())
        .unwrap();
    let (mut parts, _) = req.into_parts();
    let opt = OptionalAuthUser::from_request_parts(&mut parts, &())
        .await
        .unwrap();
    assert!(opt.is_some());
    assert_eq!(opt.0.unwrap().email, "opt@example.com");

    // 2. Absent token -> returns None without error
    let req_absent = Request::builder().body(()).unwrap();
    let (mut parts_absent, _) = req_absent.into_parts();
    let opt_absent = OptionalAuthUser::from_request_parts(&mut parts_absent, &())
        .await
        .unwrap();
    assert!(opt_absent.is_none());
}

#[tokio::test]
async fn test_pbac_permissions() {
    let secret = "test-secret-key-12345678901234567890";
    std::env::set_var("JWT_SECRET", secret);

    clear_test_role_permissions();

    // 1. Super Admin bypasses all feature permission checks
    let admin_tokens =
        generate_token_pair("1", "admin@coffe.com", "admin", secret, 15).unwrap();

    let req_admin = Request::builder()
        .header(AUTHORIZATION, format!("Bearer {}", admin_tokens.access_token))
        .body(())
        .unwrap();
    let (mut parts_admin, _) = req_admin.into_parts();
    let perm_res = RequireSupplierDelete::from_request_parts(&mut parts_admin, &()).await;
    assert!(perm_res.is_ok(), "Super Admin should bypass supplier delete check");

    // 2. Setup Barista (roleId = 3) permissions:
    // Can view & create purchase orders, but cannot delete suppliers and cannot view finance reports
    let mut barista_perms = HashMap::new();
    barista_perms.insert(
        "purchase_order".to_string(),
        PermissionAction {
            view: true,
            create: true,
            edit: false,
            delete: false,
        },
    );
    barista_perms.insert(
        "supplier".to_string(),
        PermissionAction {
            view: false,
            create: false,
            edit: false,
            delete: false,
        },
    );
    barista_perms.insert(
        "expense".to_string(),
        PermissionAction {
            view: false,
            create: false,
            edit: false,
            delete: false,
        },
    );
    barista_perms.insert(
        "finance".to_string(),
        PermissionAction {
            view: false,
            create: false,
            edit: false,
            delete: false,
        },
    );
    set_test_role_permissions(3, barista_perms);

    let barista_tokens =
        generate_token_pair_with_role_id("3", "barista@coffe.com", "barista", Some(3), secret, 15)
            .unwrap();

    // 2a. Barista can view purchase orders
    let req_po_view = Request::builder()
        .header(AUTHORIZATION, format!("Bearer {}", barista_tokens.access_token))
        .body(())
        .unwrap();
    let (mut parts_po_view, _) = req_po_view.into_parts();
    let po_view_res = RequirePurchaseView::from_request_parts(&mut parts_po_view, &()).await;
    assert!(po_view_res.is_ok(), "Barista should be allowed to view purchase orders");

    // 2b. Barista can create purchase orders
    let req_po_create = Request::builder()
        .header(AUTHORIZATION, format!("Bearer {}", barista_tokens.access_token))
        .body(())
        .unwrap();
    let (mut parts_po_create, _) = req_po_create.into_parts();
    let po_create_res = RequirePurchaseCreate::from_request_parts(&mut parts_po_create, &()).await;
    assert!(po_create_res.is_ok(), "Barista should be allowed to create purchase orders");

    // 2c. Barista CANNOT delete suppliers (403 Forbidden)
    let req_sup_del = Request::builder()
        .header(AUTHORIZATION, format!("Bearer {}", barista_tokens.access_token))
        .body(())
        .unwrap();
    let (mut parts_sup_del, _) = req_sup_del.into_parts();
    let sup_del_res = RequireSupplierDelete::from_request_parts(&mut parts_sup_del, &()).await;
    assert!(sup_del_res.is_err(), "Barista should be forbidden from deleting suppliers");

    // 2d. Barista CANNOT view financial overview (403 Forbidden)
    let req_fin = Request::builder()
        .header(AUTHORIZATION, format!("Bearer {}", barista_tokens.access_token))
        .body(())
        .unwrap();
    let (mut parts_fin, _) = req_fin.into_parts();
    let fin_res = RequireFinanceView::from_request_parts(&mut parts_fin, &()).await;
    assert!(fin_res.is_err(), "Barista should be forbidden from viewing financial reports");

    clear_test_role_permissions();
}
