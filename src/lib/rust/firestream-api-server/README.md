# firestream-api-server

This README explains how to collaborate on this Gerust application.

The example application implements a simple tasks management system. [Tasks](./db/src/entities/tasks.rs) are stored in PostgreSQL and can be [created, read, updated, and deleted](./web/src/controllers/tasks.rs) via the web interface. Any writing operations [require authentication via a token](./web/src/middlewares/auth.rs).

## Prerequisites

* Rust (install via [rustup](https://rustup.rs))
* [Docker](https://www.docker.com)

## Project Structure

Distinct parts of the project are separated into separate crates:

```
.
├── cli    // CLI tools forrunning DB migrations or generating project files
├── config // Defines the `Config` struct and handles building the configuration from environment-specific TOML files and environment variables
├── db     // Encapsulates database access, migrations, as well as entity definitions and related code (if the project uses a database)
├── macros // Contains macros for application tests
└── web    // The web interface as well as tests for it
```

### Environment

The project uses `.env` and `.env.test` files to store configuration settings for the development and test environments respectively. Those files are read automatically by the parts of the application that require configuration settings to be present.

A Docker setup with preconfigured databases for the development and test environments is created out-of-the-box with the project. Boot the containers with

```
docker compose up
```

The `.env` and `.env.test` contain matching configuration out-of-the-box.

## Commands

Running the application in development mode:

```
cargo run
```

Running the application tests:

```
cargo test
```

Running database tasks like executing migrations or seeding the database (see the [CLI create](./cli/README.md) for detailed documentation):

```
cargo db
```

Generating project files like entities, controllers, tests, etc. (see the [CLI create](./cli/README.md) for detailed documentation):

```
cargo generate
```

Building the project's docs:

## Building documentation

Build the project's documentation with:

```
cargo doc --workspace --all-features
```
