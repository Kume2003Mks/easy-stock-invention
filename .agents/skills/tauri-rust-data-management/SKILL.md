---
name: tauri-rust-data-management
description: Guidelines for managing data with Tauri and Rust backend for POS system, including SQLite optimization, multi-thread connection management, schema migration, error handling, offline-first and cloud sync.
---
# Skill Document: Tauri + Rust Backend for POS System (Offline-First & Cloud Sync)

Managing the database on the backend with Rust is an excellent choice for a system that requires maximum performance like a POS system.

## 1. Optimizing SQLite for Speed and SSD Preservation

* Using the `rusqlite` library in Rust helps reduce the overhead of sending data back and forth between the Frontend and Backend, maximizing CPU and disk performance.
* `PRAGMA` commands should be executed immediately upon opening a connection to tune the SQLite engine and reduce disk writes.
* Setting `journal_mode = WAL` allows the database to write to a separate log file, drastically reducing SSD write cycles.
* Setting `synchronous = NORMAL` ensures data transactions don't wait for disk synchronization every time, preventing the Read operations from being blocked while Write operations are processing.
* Setting `temp_store = MEMORY` moves temporary file creation and data sorting to RAM, which is significantly faster and helps preserve SSD lifespan.
* For saving multiple records, commands should be grouped into a single Transaction (Batch Writing) so data is written to the SSD only once upon COMMIT.

## 2. Connection Management and Multi-thread System

* Since Tauri's architecture operates in a multi-threaded manner.
* SQLite has a limitation allowing only 1 Write connection at a time.
* It is necessary to wrap the SQLite Connection in a `Mutex` to ensure thread-safe usage.
* Once wrapped in a `Mutex`, it can be stored in Tauri's State system for safe concurrent database access.

## 3. Schema Migration Management

* Separating database schema commands into a separate `.sql` file keeps the Rust code clean and makes version control easier.
* Create a `schema.sql` file and use `CREATE TABLE IF NOT EXISTS` commands to prevent errors during repeated application startups.
* The `include_str!` macro can be used to read and embed text from the `.sql` file into the binary during the compilation process.
* Executing table creation commands can be done all at once via `rusqlite`'s `execute_batch` function.
* When separating the Schema file, it is crucial to include the command `PRAGMA foreign_keys = ON;` so data constraints (like ON DELETE RESTRICT rules) function completely.

## 4. Error Handling Architecture

* The best practice for error handling is returning standardized machine-readable Error Codes (e.g., `ERR_UNAUTHORIZED`, `ERR_PRINTER_OFFLINE`, `ERR_UNIQUE_CONSTRAINT`) from the Backend.
* Avoid returning raw arbitrary English messages or direct database error strings.
* In Rust, an `enum` can be used to clearly define and organize these Error Code sets.
* Integrate with `serde::Serialize` to allow Tauri's system to convert Error Codes into a structured JSON format (e.g., `{ "code": "ERR_..." }`) and return them to the Frontend.
* The Frontend is responsible for intercepting these error codes and translating them into user-friendly localized UI alerts.
* This method hides internal implementation details from end users while providing reliable error discrimination for both frontend logic and developer debugging.

## 5. Offline-First and Cloud Sync Architecture

To ensure the POS system runs smoothly even without internet and can back up data to the Cloud when ready, the database architecture here relies on Rust combined with SQLite.

| Component | Best Practice |
| :--- | :--- |
| **Local as Source of Truth** | Always read and write all data to the local SQLite first to ensure storefront sales speed isn't delayed by internet speed. |
| **UUID v7** | Enforce UUID v7 as the Primary Key in all tables to prevent ID collision when syncing data from multiple POS branches to a central Cloud Database. |
| **Sync Queue / Change Tracking** | Use a dedicated `sync_queue` table (`id`, `entity_type`, `entity_id`, `action`, `payload`, `sync_status`, `created_at`) to track records waiting for upload (`PENDING`, `SYNCED`, `FAILED`). Keep this decoupled from `activity_logs` which serves exclusively as an immutable RBAC Audit Trail. |
| **Background Worker in Rust** | Use an Async Runtime like `tokio` to run a Background Task on the Rust side, separate from the main UI, to continuously poll `PENDING` data from `sync_queue`, send it to the Cloud API (via libraries like `reqwest`), and update the status to `SYNCED` upon success. |
| **Conflict Resolution** | If fetching updated data from the Cloud to the Local machine, use the `updated_at` field for Timestamp-based Resolution to verify which data is newer, preventing overwriting of the most recently modified data. |
