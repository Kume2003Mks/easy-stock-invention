---
name: tauri-clean-architecture
description: Guidelines for implementing Clean Architecture on a Tauri + Rust backend for an offline-first POS system.
---

# Clean Architecture in Tauri + Rust Backend

## 🎯 Architectural Goal
The primary objective of implementing Clean Architecture on a Tauri + Rust backend for an offline-first POS system is the Separation of Concerns. By decoupling the core business rules from external technologies (such as the Tauri UI window, the SQLite local storage, and internet networks) , we achieve a highly modular system that prevents code mutation, ensures long-term maintainability, and makes automated testing incredibly fast and reliable.

## 📐 The 4 Layers of Architecture
Source code inside the `src-tauri/src/` directory is strictly divided into layers based on data flow boundaries:

### 1. Domain Layer (Core Business Rules)
The innermost layer that encapsulates the data structures (Entities) and enterprise business rules that do not change based on technical stack upgrades.

**Strict Constraint:** This layer must be pure Rust. It is completely forbidden to import any infrastructure-driven frameworks or external drivers like `tauri` or `rusqlite`. (Basic serialization libraries like `serde` are permitted for data modeling structures).

*   **Entities (`domain/entities.rs`):** Contains core definitions such as `Product`, `Order`, `OrderItem`, `User`, `Role`, and `ActivityLog`, alongside internal validation and pricing logics.
*   **Interfaces (`domain/repositories.rs`):** Defines contract Traits specifying required database behaviors without stating how they work.

### 2. Use Cases / Application Layer (Business Workflows)
This layer orchestrates and directs the flow of data to and from the domain entities, implementing specific business actions (workflows).

*   **Data Flow Behavior:** Use cases are completely blind to where the data ends up (e.g., whether it saves to a local SQLite database or flies up to a cloud server). It only relies on invoking abstract behaviors like `OrderRepository.save()`.
*   **Workflows (`use_cases/checkout.rs`):** E.g., `CreateOrderUseCase`, which handles receiving a cart payload -> validating against stock availability -> calculating grand totals -> calling the repository transaction -> and pushing an operational record to the Audit Trail workflow.

### 3. Interface Adapters Layer (Data Translation Gates)
This layer translates data formats between external delivery mechanisms (e.g., Frontend UI JSON structures) into a format convenient for the Use Cases layer.

*   **Adapters (`adapters/commands.rs`):** Houses the `#[tauri::command]` functions hooked into the Tauri IPC bridge.
*   **Core Responsibilities:** Receives Data Transfer Objects (DTOs) from the Svelte frontend JSON inputs, screens active user role permissions (e.g., ensuring the caller is a Cashier or Manager), passes the raw request data down to the targeted Use Case, catches internal failures, and maps them out as standardized JSON Error Codes back to the frontend.

### 4. Infrastructure Layer (External Tools & Technologies)
The outermost layer where volatile details like operational databases, network clients, frameworks, and operating system tools live.

**Persistence Structures (`adapters/persistence/`):**
*   Configures the SQLite native connection and runs PRAGMA optimization rules via the `rusqlite` library.
*   Executes concrete SQL raw statements and manages manual database transaction locks.
*   Reads the root `schema.sql` file at compile time via `include_str!` to run initial DDL migrations.
*   Drives the `tokio`-managed asynchronous background synchronization workers, picking up rows with a PENDING status to perform safe up-stream payloads via `reqwest`.

## 💡 Practical Benefits for an Offline-First POS System

*   **Flexibility for Testing (Frictionless Mocking):** Because Use Cases communicate exclusively through traits, automated integration test scripts in the `tests/` directory can instantly instantiate a simple in-memory vector `MockOrderRepository`. This allows validation of complex business calculations and rollback workflows without making actual I/O connections to SQLite files or spinning up active Tauri frame engines.
*   **Decoupled Background Sync (Non-Blocking UI):** The `tokio` background task acts directly upon `SyncDataUseCase` through simple dependency injections. It functions independent of the Tauri event cycle, ensuring that even if network packets are lagging or local data sync workloads are peaking, the cashier's front-facing checkout page remains completely fluid and responsive.
*   **High Maintainability (Future-Proofing):** If you ever choose to migrate the on-device database from SQLite to a different embedded key-value store, or swap the network request library from `reqwest` to another package, the core billing logic, inventory algorithms, and audit trail logs will require zero changes. The outermost infrastructure layer absorbs the impact entirely, leaving the core business engine unharmed.
