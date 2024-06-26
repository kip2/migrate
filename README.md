[English](README.md)|[日本語](README-ja.md)

<h1 align="center">migrate</h1>

<h2 align="center">A Simple Migration App</h2>

This is a simple migration application that can be executed from the console.

# Prerequisites

- This application assumes the use of MySQL and PostgreSQL.
- Database configuration is done through a .env file.
- The application is not distributed as a binary file, so you will need to build it in your environment.

# Environment Setup

## How to Build

Execute the following in an environment where Rust is installed:

```shell
cargo build --release
```

Please deploy the built files to the directory.

The built file is generated at the following path:

```shell
./target/release/migrate
```

## DB Access Configuration

Setting up the database access information is necessary.

Please follow the steps below:

1. Place the .env file in the same directory as the built migrate executable.
2. Write your DB access settings in the .env file in the following format:

```env
# mysql
DATABASE_URL=mysql://username:password@hostname:port/db_name

# postgres
DATABASE_URL=postgres://username:password@hostname:port/db_name
```

Automatically determines the database to connect from the description in the `.env` file and connects.

# Preliminary Steps

## Creating a Migration Management Table

First, create a table to manage the migrations.

Before executing, make sure that you have access to Database and have set up the connection in the `.env` file.

```shell
./migrate -i

# or
./migrate --init
```

After executing the command, a table named `_migrations` for managing migrations will be created in the DB.

## Creating Migration Files

Create files to define the migrations you want to execute.

```shell
./migrate -c

# or
./migrate --create
```

After executing the command, files like the following will be created:

```shell
# up file
./Migrations/<YYYY-MM-DD>_<UNIX_TIME_STAMP>_up.sql
# ./Migrations/2000-01-01_1234567890_up.sql

# down file
./Migrations/<YYYY-MM-DD>_<UNIX_TIME_STAMP>_down.sql
# ./Migrations/2000-01-01_1234567890_down.sql
```

## Configuring the Migrations to Execute

In the `up file` and `down file` created, specify the migrations you want to execute.

### up file

Specify the migrations you want to execute.

Any statements that can be executed as SQL can be included.

It is also possible to describe multiple queries.

When writing, make sure to end each SQL statement with a `;`.

For example, you might write an SQL statement to create a table like this:

```sql
CREATE TABLE users (
                id BIGINT PRIMARY KEY AUTO_INCREMENT,
                username VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL UNIQUE,
                password VARCHAR(255) NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### down file

Write SQL statements to rollback the execution results of the SQL defined in the up file.

For example,

- If you define table creation in `up file`, then in `down file`, define an SQL statement to delete the table created in `up file`.
- If you define an insert statement in `up file`, then in `down file`, define an SQL statement to delete the data inserted in `up file`.

In the following example, we define an SQL statement that drops the table created in the `up file` section.

```sql
DROP TABLE users;
```

# Executing Migrations

After the environment setup and pre-preparation are complete, the migration will be executed with the following command.

```shell
./migrate
```

## Migration Target Files

Past migrations are managed in the `_migrations` table.

If you add a new migration file, only the newly added migration will be executed.

Here's an example procedure:

```shell
# Execute the migration
./migrate

# Create a new migration file
./migrate --create

# Add SQL statements to the migration file
# Details are omitted

# Execute the new migration
# Only the newly added migration will be executed
./migrate
```

# Rollback

It is possible to rollback the performed migrations to a specific stage.

The command is as follows:

```shell
# <n> specifies how many stages to rollback
./migrate -r <n>
# or
./migrate --rollback <n>

# For example
# To rollback to 2 stages before
./migrate -r 2
# or
./migrate --rollback 2
```

Also, it will only execute the possible number of rollbacks.

For a DB where 2 migrations have been performed, if a number greater than 2 is specified, only 2 rollbacks will be executed (even if 10 or 1000 is specified, only 2 will be executed).

# Help

If you encounter any issues with the command, please refer to the help documentation.

You can view the help with the following command:

```shell
./migrate -h
# or
./migrate --help
```

# LICENSE

[MIT LICENSE](https://github.com/kip2/sqcr/blob/main/LICENSE)

# AUTHOR

[kip2](https://github.com/kip2)
