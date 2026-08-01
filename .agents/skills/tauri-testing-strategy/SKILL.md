---
name: tauri-testing-strategy
description: Guidelines for Multi-layered Testing Strategy (Tauri POS) covering both Rust backend and Svelte frontend.
---

# Multi-layered Testing Strategy (Tauri POS)

This document outlines the concepts, design, and tools for a Multi-layered Testing strategy to guarantee stability, security, and peak performance for the point-of-sale system.

```text
   [ Frontend: Vitest / Playwright ] 
                   │
           (Tauri IPC Bridge)
                   ▼
[ Backend: cargo test + Tokio Async + SQLite In-Memory ]
```

## 🛠️ 1. Backend Testing (Rust)

Backend testing will utilize the standard `cargo test` suite alongside async and serialization tools. By leveraging Clean Architecture, we can test Business Logic independently without launching the actual Tauri application window.

### 🧪 Unit Testing (Pure Rust - Domain & Use Cases Layer)

*   **Error Code & Serialization:**
    *   **Objective:** Ensure system error enums serialize correctly to JSON via `serde::Serialize` before sending them to the frontend.
    *   **Approach:** Write test asserts to verify that when an error occurs, the resulting JSON always includes standard codes (e.g., `ERR_INTERNAL` or `ERR_UNIQUE_CONSTRAINT`).
*   **Helper Functions (UUID v7):**
    *   **Objective:** Verify utilities, such as extracting timestamps from a UUID v7 structure.
    *   **Approach:** Mock extraction and compare to ensure the parsed timestamp matches the actual record creation time.
*   **Mock Repository (Domain Logic):**
    *   **Objective:** Test core business logic (e.g., total bill calculations) without relying on a real database.
    *   **Approach:** Since Use Cases communicate through Interfaces (Trait), we can create a `MockOrderRepository` in the test file that simply saves data to a RAM array/vector (`Vec`), making tests fast and safe.

### 🧪 Integration Testing (Database & Transaction Logic - Infrastructure Layer)

This tests the actual SQLite connections via the `rusqlite` library to verify SQL Queries and state management.

*   **In-Memory Database:** Use `Connection::open_in_memory()` to open temporary RAM databases instead of writing to disk, ensuring fast execution and preserving SSD lifespan on dev machines and CI/CD servers.
*   **Schema Migration:** Verify that `schema.sql` (embedded at compile time via the `include_str!` macro) runs smoothly via `execute_batch` without syntax errors and successfully creates all required tables (`users`, `products`, `orders`, `activity_logs`).
*   **Transaction Atomicity (Create Order):**
    *   **Test Case 1 (Success):** Mock an order creation. Data must successfully save across all 3 tables (`orders`, `order_items`, `activity_logs`), and products stock must decrease correctly.
    *   **Test Case 2 (Rollback):** Intentionally fail the command mid-way (e.g., adding a non-existent item) and assert that no data is saved in the main or log tables (100% Rollback), effectively preventing data corruption.
*   **Asynchronous & Offline-First:**
    *   Use the `#[tokio::test]` macro to start an async runtime for testing the sync queue.
    *   **Change Tracking:** Upon modifying data, the system must automatically adjust the `sync_status` column in the activity table to `PENDING`.
    *   **Conflict Resolution:** Simulate network recovery. Rust must compare the `updated_at` field (Timestamp-based Resolution) and retain the latest data to prevent overwriting new data with old data.

## 🌐 2. Frontend Testing (Svelte + TS)

This focuses on UI Logic, State Management, and user behavior by isolating the IPC Bridge commands into `src/lib/services/ipc.ts`, making frontend mocking frictionless.

### 🧪 Unit Testing & State Management Testing

Use fast tools like Vitest or Jest that integrate seamlessly with Vite.

*   **Error Handler (JSON to UI Mapping):** Send a mock error JSON object from the Backend (e.g., `{ "code": "ERR_UNIQUE_CONSTRAINT" }`) and verify that the frontend translation function properly changes the UI state to display a localized warning message like "This data already exists in the system".
*   **Pagination Logic:** Verify the pagination calculator precisely processes `LIMIT` and `OFFSET` values before dispatching commands across the IPC Bridge, preventing massive single-request data loads.

### 🧪 Integration Testing & Mocking Tauri IPC

Using `@tauri-apps/api/mocks` (`mockIPC`): Intercepts and mocks return values (Resolve/Reject) of invoke commands. This allows scripts to test POS screen behaviors like the Order Creation Loop (add item -> calc total -> pay) efficiently without relying on a background SQLite database.

*   **Test Case (Success):** Upon a successful invoke, the frontend cart must clear to zero and route to the payment confirmation screen.
*   **Test Case (Fail):** If the invoke is rejected, the screen must display an Error message, and the cart items must remain intact.

### 🧪 End-to-End (E2E) & Visual Component Testing

Test on simulated environments to verify user behavior from start to finish using Playwright alongside Storybook.

*   **Visual UI Component (Storybook):** Mock isolated components like product boxes to verify edge states, such as 0 stock or when a product is "selected to have no image." The system must correctly render a placeholder box without breaking the web layout or throwing blocking errors.
*   **Network Offline (Playwright):** Use Playwright to simulate a network disconnection (Offline). Attempt to void a bill or add an item, and verify that the Cloud Sync Status correctly displays a `PENDING` warning indicator, adhering to Offline-First conditions.

## 📊 Testing Tech Stack Summary

| Test Level | Backend (Rust) | Frontend (Svelte + TS) |
| :--- | :--- | :--- |
| **Unit Test** | `cargo test` + `serde_json` | Vitest / Jest + State Management (Context/Store) |
| **Integration Test** | `rusqlite` (In-Memory) + `tokio::test` | Vitest + `@tauri-apps/api/mocks` (`mockIPC`) |
| **E2E / UI Test** | — | Playwright (Easy Online/Offline toggle) + Storybook |
