mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("db/migrations");
}
pub enum DbKind {
    Postgres,
}

pub async fn migrate(db_kind: DbKind) {
    match db_kind {
        DbKind::Postgres => {
            println!("Running DB migrations...");
            let (mut client, con) = tokio_postgres::connect(
                crate::global::DATABASE_URL.as_str(),
                tokio_postgres::NoTls,
            )
            .await
            .expect("Failed to create a connection to the database");

            tokio::spawn(async move {
                if let Err(e) = con.await {
                    eprintln!("connection error: {}", e);
                }
            });
            let migration_report = embedded::migrations::runner()
                .run_async(&mut client)
                .await
                .expect("Failed to run database migrations");

            for migration in migration_report.applied_migrations() {
                println!(
                    "Migration Applied -  Name: {}, Version: {}",
                    migration.name(),
                    migration.version()
                );
            }

            println!("DB migrations finished!");
        }
    }
}
