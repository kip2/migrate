[English](README.md)|[日本語](README.ja.md)

<h1 align="center">migrate</h1>

<h2 align="center">A Simple Migration App</h2>

This is a simple migration application that can be executed from the console.

# Prerequisites

- This application assumes the use of MySQL.
- Database configuration is done through a .env file.
- The application is not distributed as a binary file, so you will need to build it in your environment.

# Environment Setup

Setting up the database access information is necessary.

Please follow the steps below:

1. Place the .env file in the same directory as the built migration executable.
2. Write your DB access settings in the .env file in the following format:

```env
DATABASE_URL=mysql://username:password@hostname:port/db_name
```

# Preliminary Steps

## Creating a Migration Management Table

First, create a table to manage the migrations.

Before executing, make sure that you have access to MySQL and have set up the connection in the `.env` file.

```shell
migrate -i

# or
migrate --init
```

After executing the command, a table named `migrations` for managing migrations will be created in the DB.

## Creating Migration Files

Create files to define the migrations you want to execute.

```shell
migrate -c

# or
migrate --create
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

- If you define table creation in `up`, then in `down`, define an SQL statement to delete the table created in `up`.
- If you define an insert statement in `up`, then in `down`, define an SQL statement to delete the data inserted in `up`.

In the following example, the table defined in `up` is being deleted.

```sql
DROP TABLE users;
```

# Executing Migrations

Once the environment setup and preliminary steps are completed, execute the following command to perform the migration.

```shell
migrate
```

## Migration Target Files

Past migrations are managed in the `migrations` table.

If you add a new migration file, only the newly added migration will be executed.

Here's an example procedure:

```shell
# Execute the migration
migrate

# Create a new migration file
migrate --create

# Add SQL statements to the migration file
# Details are omitted

# Execute the new migration
# Only the newly added migration will be executed
migrate
```

# Rollback

It is possible to rollback the performed migrations to a specific stage.

The command is as follows:

```shell
# <n> specifies how many stages to rollback
migrate -r <n>
# or
migrate --rollback <n>

# For example
# To rollback to 2 stages before
migrate -r 2
# or
migrate --rollback 2
```

Also, it will only execute the possible number of rollbacks.

For a DB where 2 migrations have been performed, if a number greater than 2 is specified, only 2 rollbacks will be executed (even if 10 or 1000 is specified, only 2 will be executed).

# Help

If you are unsure about what commands are available, please refer to the help.

You can view the help with the following command:

```shell
migrate -h
# or
migrate --help
```

# LICENSE

[MIT LICENSE](https://github.com/kip2/sqcr/blob/main/LICENSE)

# AUTHOR

[kip2](https://github.com/kip2)
