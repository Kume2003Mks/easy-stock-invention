PRAGMA foreign_keys = ON;

-- 1. ตารางหมวดหมู่สินค้า
CREATE TABLE IF NOT EXISTS Categories (
    category_id TEXT PRIMARY KEY, 
    name TEXT NOT NULL
);

-- 2. ตารางผู้จัดจำหน่าย / ร้านส่ง
CREATE TABLE IF NOT EXISTS Suppliers (
    supplier_id TEXT PRIMARY KEY, 
    name TEXT NOT NULL,
    contact_info TEXT
);

-- 3. ตารางสินค้า (Master Data)
CREATE TABLE IF NOT EXISTS Products (
    product_id TEXT PRIMARY KEY,
    barcode TEXT UNIQUE, 
    name TEXT NOT NULL,
    category_id TEXT, 
    supplier_id TEXT NULL,              
    cost_price REAL DEFAULT 0,          
    selling_price REAL NOT NULL,        
    wholesale_price REAL DEFAULT 0,     
    current_stock INTEGER DEFAULT 0,
    reorder_level INTEGER DEFAULT 10, 
    FOREIGN KEY (category_id) REFERENCES Categories(category_id),
    FOREIGN KEY (supplier_id) REFERENCES Suppliers(supplier_id)
);

-- 4. ตารางความเคลื่อนไหวสต๊อก (Audit Trail)
CREATE TABLE IF NOT EXISTS Stock_Transactions (
    transaction_id TEXT PRIMARY KEY,
    product_id TEXT, 
    transaction_type TEXT NOT NULL, -- 'IN', 'OUT', 'ADJUST'
    quantity INTEGER NOT NULL,
    reference_no TEXT, 
    transaction_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES Products(product_id)
);
