use crate::file::get_all_migration_files;
use crate::parser::parse_text;
use crate::Migrations;
use dotenv::dotenv;
use sqlx::any::{install_default_drivers, AnyQueryResult};
use sqlx::{AnyPool, Row};
use std::io;
use std::{env, error::Error, fs};

pub async fn migrate() -> Result<(), Box<dyn Error>> {
    println!("Start migration");
    let pool = db_pool().await.unwrap();

    let last_migration = get_last_migration(&pool, Migrations::UP).await;
    let dir = "./Migrations";

    let all_up_migrations =
        get_all_migration_files(dir, Migrations::UP).expect("Failed get all migration files");
    let all_down_migrations =
        get_all_migration_files(dir, Migrations::DOWN).expect("Failed get all migration file");

    let start_index = match last_migration {
        Some(filename) => {
            all_up_migrations
                .iter()
                .position(|m| m == &filename)
                .unwrap_or(0)
                + 1
        }
        None => 0,
    };

    for (index, up_filename) in all_up_migrations.iter().enumerate().skip(start_index) {
        println!("Processing up migration for {}", &up_filename);

        let up_path = format!("{}/{}", &dir, &up_filename);
        let down_filename = all_down_migrations
            .get(index)
            .expect("Matching down migration not found");

        let queries =
            read_sql_file(&up_path).expect(&format!("Failed to read {} file", &up_filename));

        if let Err(e) = execute_queries(&pool, queries).await {
            eprintln!("Failed to up migration in {}", up_filename);
            return Err(e);
        };

        insert_migration(&pool, up_filename.clone(), down_filename.clone())
            .await
            .expect("Failed to register file in the migration table");
    }
    println!("Migration ended...");
    Ok(())
}

pub async fn create_migration_table() {
    // Table definitions for managing migrations
    dotenv().expect("Fialed to read .env file");
    let database_url = env::var("DATABASE_URL").expect("DATBASE_URL must be set");

    let query =
        if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            "CREATE TABLE IF NOT EXISTS _migrations (
        id SERIAL PRIMARY KEY,
        up_file VARCHAR(400) NOT NULL,
        down_file VARCHAR(400) NOT NULL
        );"
        } else if database_url.starts_with("mysql://") {
            "CREATE TABLE IF NOT EXISTS _migrations (
        id INT AUTO_INCREMENT PRIMARY KEY,
        up_file VARCHAR(400) NOT NULL,
        down_file VARCHAR(400) NOT NULL
        );"
        } else {
            panic!("Unsuported database type");
        };

    run(query.to_string())
        .await
        .expect("Failed migration table");
}

async fn get_last_migration(db: &AnyPool, column_type: Migrations) -> Option<String> {
    let query = format!("SELECT up_file, down_file FROM _migrations ORDER BY id DESC LIMIT 1");
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
    db: &AnyPool,
    up_file_name: String,
    down_file_name: String,
) -> Result<AnyQueryResult, Box<dyn Error>> {
    let database_url = env::var("DATABASE_URL").expect("DATBASE_URL must be set");

    let query =
        if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            "INSERT INTO _migrations (up_file, down_file) VALUES ($1, $2)"
        } else if database_url.starts_with("mysql://") {
            "INSERT INTO _migrations (up_file, down_file) VALUES (?, ?)"
        } else {
            return Err("".into());
        };

    let result = sqlx::query(query)
        .bind(up_file_name)
        .bind(down_file_name)
        .execute(db)
        .await;

    result.map_err(|e| e.into())
}

pub async fn roolback(n: u64) -> Result<(), Box<dyn Error>> {
    println!("Rolling back {} migration(s)...", n);
    let pool = db_pool().await.unwrap();
    let last_migration = get_last_migration(&pool, Migrations::DOWN).await;
    let dir = "./Migrations";
    let mut all_down_migrations =
        get_all_migration_files(dir, Migrations::DOWN).expect("Failed get all migration files");

    all_down_migrations.sort_by(|a, b| b.cmp(a));

    let start_index = match last_migration.clone() {
        Some(filename) => all_down_migrations
            .iter()
            .position(|m| m == &filename)
            .unwrap_or(0),
        None => all_down_migrations.len(),
    };

    if start_index == all_down_migrations.len() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Could not find the last migration ran: {}",
                last_migration.unwrap()
            ),
        )));
    }

    for (index, down_filename) in all_down_migrations
        .iter()
        .enumerate()
        .skip(start_index)
        .take(n.try_into().unwrap())
    {
        println!("Processing down migration for {}", &down_filename);
        let down_path = format!("{}/{}", &dir, &down_filename);
        let down_filename = all_down_migrations
            .get(index)
            .expect("Matching down migration not found");
        let queries =
            read_sql_file(&down_path).expect(&format!("Failed to read {} file", &down_filename));
        if let Err(e) = execute_queries(&pool, queries).await {
            eprintln!("Failed to up migration in {}", down_filename);
            return Err(e);
        };
        remove_migration(&pool, down_filename.clone())
            .await
            .expect("Delete execute failed");
    }

    println!("Rollback completed.");

    Ok(())
}

pub async fn remove_migration(
    db: &AnyPool,
    down_filename: String,
) -> Result<AnyQueryResult, Box<dyn Error>> {
    let database_url = env::var("DATABASE_URL").expect("DATBASE_URL must be set");

    let query =
        if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            "DELETE FROM _migrations WHERE down_file = $1"
        } else if database_url.starts_with("mysql://") {
            "DELETE FROM _migrations WHERE down_file = ?"
        } else {
            return Err("".into());
        };

    let result = sqlx::query(query).bind(down_filename).execute(db).await;

    result.map_err(|e| e.into())
}

pub async fn run(query: String) -> Result<(), Box<dyn Error>> {
    let pool = db_pool().await.unwrap();
    execute_query(&pool, query).await;
    Ok(())
}

async fn db_pool() -> Result<AnyPool, Box<dyn Error>> {
    dotenv().expect("Fialed to read .env file");
    let database_url = env::var("DATABASE_URL").expect("DABASE_URL must be set");

    install_default_drivers();

    let pool = AnyPool::connect(&database_url).await?;
    Ok(pool)
}

pub async fn read_and_run(path: String) -> Result<(), Box<dyn Error>> {
    let pool = db_pool().await.unwrap();

    // Read SQL queries
    let queries = read_sql_file(&path).unwrap();

    if let Err(e) = execute_queries(&pool, queries).await {
        eprintln!("Failed to query");
        return Err(e);
    };
    Ok(())
}

fn read_sql_file(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;

    let database_url = env::var("DATABASE_URL").expect("DATBASE_URL must be set");

    if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
        let queries = parse_text(&contents);
        Ok(queries)
    } else if database_url.starts_with("mysql://") {
        let queries = contents
            .split(';')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect();
        Ok(queries)
    } else {
        return Err("Unsupported databse".into());
    }
}

async fn execute_select_query(
    db: &AnyPool,
    query: String,
) -> Result<Vec<sqlx::any::AnyRow>, Box<dyn Error>> {
    // Check if the query string starts with SELECT
    if !query.trim_start().to_uppercase().starts_with("SELECT") {
        return Err("Query must start with SELECT".into());
    }

    let result = sqlx::query(&query).fetch_all(db).await;

    match result {
        Ok(rows) => Ok(rows),
        Err(e) => {
            println!("Database query failed: {}", e);
            Err(e.into())
        }
    }
}

async fn execute_query(db: &AnyPool, query: String) {
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

async fn execute_queries(db: &AnyPool, queries: Vec<String>) -> Result<(), Box<dyn Error>> {
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
                return Err(Box::new(e));
            }
        }
    }

    // transaction commit
    let _ = tx.commit().await.unwrap_or_else(|e| {
        println!("{:?}", e);
    });

    Ok(())
}

pub async fn get_executable_query_count(n: u64) -> u64 {
    let pool = db_pool().await.unwrap();
    let query = "SELECT COUNT(*) FROM _migrations".to_string();
    let count: u64 = get_count(&pool, query).await.expect("Not found data") as u64;

    if n > count {
        count
    } else {
        n
    }
}

async fn get_count(db: &AnyPool, query: String) -> Result<i64, Box<dyn Error>> {
    let rows = execute_select_query(&db, query).await?;
    if let Some(row) = rows.first() {
        let count: i64 = row.get(0);
        Ok(count)
    } else {
        Err("No count result found".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_get_executable_query_count() {
        let result = get_executable_query_count(100).await;
        assert_eq!(result, 2);
    }

    #[tokio::test]
    async fn test_get_count() {
        let pool = db_pool().await.unwrap();
        let query = "SELECT COUNT(*) FROM _migrations".to_string();
        let count = get_count(&pool, query).await;
        assert!(count.is_ok());
    }

    #[tokio::test]
    async fn test_remove_migration() {
        let pool = db_pool().await.unwrap();
        let down_file = "2024-04-03_1712147726_down.sql".to_string();
        let _ = remove_migration(&pool, down_file).await;
    }

    #[tokio::test]
    async fn test_migrate() {
        let _ = migrate().await;
    }

    #[tokio::test]
    async fn test_get_last_migration() {
        let pool = db_pool().await.unwrap();
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
        let pool = db_pool().await.unwrap();
        let up_file = "2024-03-31_1711885799_up.sql".to_string();
        let down_file = "2024-03-31_1711885799_down.sql".to_string();
        let _ = insert_migration(&pool, up_file, down_file).await;
    }

    #[tokio::test]
    async fn test_select_query() {
        let pool = db_pool().await.unwrap();
        let query = "SELECT up_file FROM _migrations ORDER BY id DESC LIMIT 1".to_string();
        let result = execute_select_query(&pool, query).await;
        for row in result.unwrap() {
            let filename: String = row.get("up_file");
            println!("{:?}", filename);
        }
        let query = "DELETE FROM test1".to_string();
        let result = execute_select_query(&pool, query).await;
        assert!(result.is_err(), "Expected an error for non-SELECT query");
    }
}
