use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool, Row};
use std::{env, error::Error, fs};

pub async fn create_migration_table() {
    // Table definitions for managing migrations
    let query = "CREATE TABLE migrations (
        id INT AUTO_INCREMENT PRIMARY KEY,
        filename VARCHAR(400)
    );"
    .to_string();

    run(query).await.expect("Failed migration table");
}

async fn get_last_migration(db: &Pool<MySql>) -> Option<String> {
    let query = format!("SELECT filename FROM migrations ORDER BY id DESC LIMIT 1");
    let result = select_query(db, query).await;

    match result {
        Ok(rows) => {
            if let Some(row) = rows.first() {
                let filename: String = row.get("filename");
                Some(filename)
            } else {
                None
            }
        }
        Err(e) => {
            println!("Query failed: {}", e);
            None
        }
    }
}

pub async fn insert_migration(db: &Pool<MySql>, filename: String) -> Result<(), Box<dyn Error>> {
    let query = format!("INSERT INTO migrations (filename) VALUES ('{}')", filename);
    execute_query(db, query).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_get_last_migration() {
        let pool = db_pool().await;
        let result = get_last_migration(&pool).await;
        match result {
            Some(value) => println!("Got a value: {}", value),
            None => println!("Got nothing"),
        }
    }

    #[tokio::test]
    async fn test_insert_migration() {
        let pool = db_pool().await;
        let filename = "2024-03-31_1711885797_up.sql".to_string();
        let _ = insert_migration(&pool, filename).await;
    }

    #[tokio::test]
    async fn test_select_query() {
        let pool = db_pool().await;
        let query = "SELECT filename FROM migrations ORDER BY id DESC LIMIT 1".to_string();
        let result = select_query(&pool, query).await;
        for row in result.unwrap() {
            let filename: String = row.get("filename");
            println!("{:?}", filename);
        }
    }
}

pub async fn run(query: String) -> Result<(), Box<dyn Error>> {
    let pool = db_pool().await;
    execute_query(&pool, query).await;
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

    execute_queries(&pool, queries).await;
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

async fn select_query(
    db: &Pool<MySql>,
    query: String,
) -> Result<Vec<sqlx::mysql::MySqlRow>, Box<dyn Error>> {
    let result = sqlx::query(&query).fetch_all(db).await;

    match result {
        Ok(rows) => Ok(rows),
        Err(e) => {
            println!("Database query failed: {}", e);
            Err(e.into())
        }
    }
}

async fn execute_query(db: &Pool<MySql>, query: String) {
    // Gererate transaction
    let mut tx = db.begin().await.expect("transaction error.");

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

    // transaction commit
    let _ = tx.commit().await.unwrap_or_else(|e| {
        println!("{:?}", e);
    });
}

async fn execute_queries(db: &Pool<MySql>, queries: Vec<String>) {
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
