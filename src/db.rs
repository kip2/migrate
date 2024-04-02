use crate::file::get_all_migration_files;
use crate::Migrations;
use sqlx::mysql::{MySqlPoolOptions, MySqlQueryResult};
use sqlx::{MySql, Pool, Row};
use std::{env, error::Error, fs};

pub async fn migrate() -> Result<(), Box<dyn Error>> {
    let pool = db_pool().await;
    let last_migration = get_last_migration(&pool, Migrations::UP).await;
    let dir = "./Migrations";
    let all_migrations =
        get_all_migration_files(dir, Migrations::UP).expect("Failed get all migration files");

    let start_index = match last_migration {
        Some(filename) => {
            all_migrations
                .iter()
                .position(|m| m == &filename)
                .unwrap_or(0)
                + 1
        }
        None => 0,
    };

    for filename in all_migrations.iter().skip(start_index) {
        println!("Processing up migration for {}", filename);
        let path = format!("./Migrations/{}", &filename);
        let queries = read_sql_file(&path).expect(&format!("Failed to read {} file", &filename));
        execute_queries(&pool, queries).await;
    }
    println!("Migration ended...");
    Ok(())
}

pub async fn create_migration_table() {
    // Table definitions for managing migrations
    let query = "CREATE TABLE migrations (
        id INT AUTO_INCREMENT PRIMARY KEY,
        up_file VARCHAR(400) NOT NULL,
        down_file VARCHAR(400) NOT NULL
    );"
    .to_string();

    run(query).await.expect("Failed migration table");
}

async fn get_last_migration(db: &Pool<MySql>, column_type: Migrations) -> Option<String> {
    let query = format!("SELECT up_file, down_file FROM migrations ORDER BY id DESC LIMIT 1");
    let result = execute_select_query(db, query).await;

    match result {
        Ok(rows) => {
            if let Some(row) = rows.first() {
                match column_type {
                    Migrations::UP => {
                        let filename: String = row.get("up_file");
                        Some(filename)
                    }
                    Migrations::DOWN => {
                        let filename: String = row.get("down_file");
                        Some(filename)
                    }
                }
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

pub async fn insert_migration(
    db: &Pool<MySql>,
    up_file_name: String,
    down_file_name: String,
) -> Result<MySqlQueryResult, Box<dyn Error>> {
    let query = "INSERT INTO migrations (up_file, down_file) VALUES (?, ?)";

    let result = sqlx::query(query)
        .bind(up_file_name)
        .bind(down_file_name)
        .execute(db)
        .await;

    result.map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_get_last_migration() {
        let pool = db_pool().await;
        let result = get_last_migration(&pool, Migrations::UP).await;
        match result {
            Some(value) => println!("Got a value: {}", value),
            None => println!("Got nothing"),
        }

        let result = get_last_migration(&pool, Migrations::DOWN).await;
        match result {
            Some(value) => println!("Got a value: {}", value),
            None => println!("Got nothing"),
        }
    }

    #[tokio::test]
    async fn test_insert_migration() {
        let pool = db_pool().await;
        let up_file = "2024-03-31_1711885799_up.sql".to_string();
        let down_file = "2024-03-31_1711885799_down.sql".to_string();
        let _ = insert_migration(&pool, up_file, down_file).await;
    }

    #[tokio::test]
    async fn test_select_query() {
        let pool = db_pool().await;
        let query = "SELECT filename FROM migrations ORDER BY id DESC LIMIT 1".to_string();
        let result = execute_select_query(&pool, query).await;
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

async fn execute_select_query(
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
