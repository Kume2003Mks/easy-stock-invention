PRAGMA foreign_keys = ON;

-- ==========================================================
-- 1. ตารางหมวดหมู่สินค้า (Categories)
-- ==========================================================
CREATE TABLE IF NOT EXISTS Categories (
    category_id TEXT PRIMARY KEY, 
    name TEXT NOT NULL UNIQUE
);

-- ==========================================================
-- 2. ตารางผู้จัดจำหน่าย / ร้านส่ง (Suppliers)
-- ==========================================================
CREATE TABLE IF NOT EXISTS Suppliers (
    supplier_id TEXT PRIMARY KEY, 
    name TEXT NOT NULL,
    contact_info TEXT
);

-- ==========================================================
-- 3. ตารางสินค้า Master Data (Products)
-- ==========================================================
CREATE TABLE IF NOT EXISTS Products (
    product_id TEXT PRIMARY KEY,
    barcode TEXT UNIQUE, 
    name TEXT NOT NULL,
    category_id TEXT, 
    supplier_id TEXT,              
    cost_price REAL NOT NULL DEFAULT 0.0,          
    selling_price REAL NOT NULL DEFAULT 0.0,        
    wholesale_price REAL NOT NULL DEFAULT 0.0,     
    current_stock INTEGER NOT NULL DEFAULT 0,
    reorder_level INTEGER NOT NULL DEFAULT 10,
    FOREIGN KEY (category_id) REFERENCES Categories(category_id) ON UPDATE CASCADE ON DELETE SET NULL,
    FOREIGN KEY (supplier_id) REFERENCES Suppliers(supplier_id) ON UPDATE CASCADE ON DELETE SET NULL
);

-- ==========================================================
-- 4. ตารางความเคลื่อนไหวสต๊อก Audit Trail (Stock Transactions)
-- ==========================================================
CREATE TABLE IF NOT EXISTS Stock_Transactions (
    transaction_id TEXT PRIMARY KEY,
    product_id TEXT, 
    transaction_type TEXT NOT NULL CHECK(transaction_type IN ('IN', 'OUT', 'ADJUST')),
    quantity INTEGER NOT NULL,
    reference_no TEXT, 
    transaction_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES Products(product_id) ON UPDATE CASCADE ON DELETE SET NULL
);

-- ==========================================================
-- 5. ตารางเก็บการตั้งค่าของแอปพลิเคชัน (App Settings)
-- ==========================================================
CREATE TABLE IF NOT EXISTS App_Settings (
    setting_key TEXT PRIMARY KEY,
    setting_value TEXT NOT NULL,
    description TEXT,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================================
-- 6. ดัชนีเพื่อเพิ่มประสิทธิภาพการค้นหา (Indexes)
-- ==========================================================
CREATE INDEX IF NOT EXISTS idx_products_name ON Products(name);
CREATE INDEX IF NOT EXISTS idx_products_category_id ON Products(category_id);
CREATE INDEX IF NOT EXISTS idx_products_supplier_id ON Products(supplier_id);
CREATE INDEX IF NOT EXISTS idx_stock_transactions_product_id ON Stock_Transactions(product_id);
CREATE INDEX IF NOT EXISTS idx_stock_transactions_date ON Stock_Transactions(transaction_date);
