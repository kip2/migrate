use clap::Parser;
use std::error::Error;

use crate::file::create_migration_file;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long = "create")]
    create: bool,

    #[arg(long = "init")]
    init: bool,

    #[arg(
        short = 'r',
        long = "rollback",
        help = "Rollback database",
        default_value = "0"
    )]
    rollback: u32,
}

fn get_args() -> Args {
    Args::parse()
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let args = get_args();
    println!("args: {:?}", args);

    if args.create {
        create_migration_file().expect("Failed migration files");
    } else if args.init {
        println!("init!");
    } else if args.rollback > 0 {
        println!("rollback!");
    } else {
        println!("other!");
    }

    Ok(())
}
