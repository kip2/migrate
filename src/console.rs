use clap::Parser;
use std::error::Error;

use crate::{
    db::{create_migration_table, migrate, roolback},
    file::create_migration_file,
};

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long = "create", help = "Create migrate files")]
    create: bool,

    #[arg(long = "init", help = "Create migrate table if it doesn't exist.")]
    init: bool,

    #[arg(
        short = 'r',
        long = "rollback",
        help = "Rollback database",
        default_value = "0"
    )]
    rollback: usize,
}

pub async fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.create {
        create_migration_file().expect("Failed migration files");
    } else if args.init {
        create_migration_table().await;
    } else if args.rollback > 0 {
        roolback(args.rollback)
            .await
            .expect("Failed rollback migrations");
    } else {
        migrate().await.expect("Failed migration");
    }

    Ok(())
}
