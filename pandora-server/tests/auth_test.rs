use pandora_server::{config::Config, db, entity::tenants, services::token_service};
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Set};
use uuid::Uuid;

async fn setup() -> (sea_orm::DatabaseConnection, Config, Uuid) {
    dotenvy::dotenv().ok();
    let cfg = Config::from_env().expect("config");
    let db = db::connect(&cfg.database_url).await.expect("db");

    // Encrypt a real tenant key and upsert a dev tenant
    let tenant_key_plain = "dev-test-tenant-key-32chars-long!";
    let encrypted_key = hefesto::encrypt(
        tenant_key_plain,
        &cfg.master_encryption_key,
        &cfg.master_encryption_key,
    )
    .expect("encrypt");

    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let existing = tenants::Entity::find_by_id(tenant_id)
        .one(&db)
        .await
        .expect("find");

    if let Some(t) = existing {
        let mut active: tenants::ActiveModel = t.into();
        active.encryption_key = Set(encrypted_key);
        active.update(&db).await.expect("update tenant");
    } else {
        tenants::ActiveModel {
            id: Set(tenant_id),
            name: Set("Dev Tenant".into()),
            slug: Set("dev".into()),
            encryption_key: Set(encrypted_key),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("insert tenant");
    }

    (db, cfg, tenant_id)
}

#[tokio::test]
async fn test_register_and_login() {
    let (db, cfg, tenant_id) = setup().await;

    let tenant = tenants::Entity::find_by_id(tenant_id)
        .one(&db)
        .await
        .expect("find")
        .expect("tenant exists");

    let tenant_key = hefesto::decrypt(
        &tenant.encryption_key,
        &cfg.master_encryption_key,
        &cfg.master_encryption_key,
    )
    .expect("decrypt tenant key");

    let email = "integration@pandora.dev";
    let password = "Test1234!";

    // Cleanup previous run
    let _ = db
        .execute_unprepared(&format!("SET app.tenant_id = '{tenant_id}'"))
        .await;
    let _ = db
        .execute_unprepared(&format!(
            "DELETE FROM users WHERE tenant_id = '{tenant_id}' AND email_lookup = '{}'",
            hefesto::hash_for_lookup(email, &tenant_key)
        ))
        .await;

    // Register
    let email_lookup = hefesto::hash_for_lookup(email, &tenant_key);
    let email_encrypted = hefesto::encrypt(email, &tenant_key, &cfg.master_encryption_key)
        .expect("encrypt email");
    let password_hash = hefesto::hash_password(password).expect("hash password");

    let txn = db::begin_tenant_txn(&db, tenant_id).await.expect("begin txn");
    let user = pandora_server::services::user_service::create(
        &txn,
        pandora_server::services::user_service::CreateUserInput {
            tenant_id,
            email_encrypted,
            email_lookup: email_lookup.clone(),
            password_hash,
        },
    )
    .await
    .expect("create user");
    txn.commit().await.expect("commit");

    assert_eq!(user.tenant_id, tenant_id);

    // Login — find user and verify password
    let txn = db::begin_tenant_txn(&db, tenant_id).await.expect("begin txn");
    let found = pandora_server::services::user_service::find_by_email_lookup(&txn, &email_lookup)
        .await
        .expect("find user")
        .expect("user exists");
    txn.commit().await.expect("commit");

    assert!(hefesto::verify_password(password, &found.password_hash));
    assert_eq!(found.id, user.id);

    // JWT round-trip
    let token = token_service::generate_access_token(
        user.id,
        tenant_id,
        email,
        &cfg.jwt_secret,
        cfg.jwt_expiration_minutes,
    )
    .expect("generate token");

    let claims = token_service::validate_access_token(&token, &cfg.jwt_secret)
        .expect("validate token");

    assert_eq!(claims.sub, user.id.to_string());
    assert_eq!(claims.tid, tenant_id.to_string());
    assert_eq!(claims.email, email);

    println!("✓ Register, login, JWT round-trip OK");
    println!("  Tenant ID : {tenant_id}");
    println!("  User   ID : {}", user.id);
}
