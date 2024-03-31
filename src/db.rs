use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use std::{env, error::Error, fs};

pub async fn create_migration_table() {
    // Table definitions for managing migrations
    let query = "CREATE TABLE migrations (
        id INT AUTO_INCREMENT PRIMARY KEY,
        filename VARCHAR(400)
    );";

    run(query).await.expect("Failed migration table");
}

pub async fn run(query: &str) -> Result<(), Box<dyn Error>> {
    let pool = db_pool().await;
    let queries = vec![query.to_string()];
    execute_query(&pool, queries).await;
    Ok(())
}

async fn db_pool() -> Pool<MySql> {
    dotenv::dotenv().expect("Fialed to read .env file");
    let database_url = env::var("DATABASE_URL").expect("DABASE_URL must be set");

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Cannot connect to the database"));
    pool
}

pub async fn read_and_run(path: String) -> Result<(), Box<dyn Error>> {
    let pool = db_pool().await;

    // Read SQL queries
    let queries = read_sql_file(&path).unwrap();

    execute_query(&pool, queries).await;
    Ok(())
}

fn read_sql_file(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;

    let queries = contents
        .split(';')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();
    Ok(queries)
}

async fn execute_query(db: &Pool<MySql>, queries: Vec<String>) {
    // Gererate transaction
    let mut tx = db.begin().await.expect("transaction error.");

    for query in queries {
        // Execute SQL query
        let result = sqlx::query(&query).execute(&mut *tx).await;

        match result {
            Ok(_) => {}
            Err(e) => {
                println!("Database query failed: {}", e);
                // rollback
                tx.rollback().await.expect("Transaction rollback error.");
                return;
            }
        }
    }

    // transaction commit
    let _ = tx.commit().await.unwrap_or_else(|e| {
        println!("{:?}", e);
    });
}
