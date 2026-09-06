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
-- 7. ตารางบิลขาย / บิลคืน (Orders)
--    order_type: SALE = บิลขาย, RETURN = บิลคืนสินค้า (RB)
--    status: HELD = พักบิล, COMPLETED = ชำระเงินแล้ว, VOIDED = ยกเลิก
-- ==========================================================
CREATE TABLE IF NOT EXISTS Orders (
    order_id TEXT PRIMARY KEY,
    order_no TEXT NOT NULL UNIQUE,
    order_type TEXT NOT NULL DEFAULT 'SALE' CHECK(order_type IN ('SALE', 'RETURN')),
    status TEXT NOT NULL DEFAULT 'COMPLETED' CHECK(status IN ('HELD', 'COMPLETED', 'VOIDED')),
    subtotal REAL NOT NULL DEFAULT 0.0,
    discount_amount REAL NOT NULL DEFAULT 0.0,
    total_amount REAL NOT NULL DEFAULT 0.0,
    payment_method TEXT NOT NULL DEFAULT 'CASH' CHECK(payment_method IN ('CASH', 'PROMPTPAY', 'TRANSFER')),
    paid_amount REAL NOT NULL DEFAULT 0.0,
    change_amount REAL NOT NULL DEFAULT 0.0,
    hold_name TEXT,
    note TEXT,
    original_order_id TEXT,
    order_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (original_order_id) REFERENCES Orders(order_id) ON UPDATE CASCADE ON DELETE SET NULL
);

-- ==========================================================
-- 8. ตารางรายการสินค้าในบิล (Order Items)
--    เก็บ snapshot ชื่อสินค้าและราคา ณ วันที่ขาย
-- ==========================================================
CREATE TABLE IF NOT EXISTS Order_Items (
    item_id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    product_id TEXT,
    product_name TEXT NOT NULL,
    quantity INTEGER NOT NULL CHECK(quantity > 0),
    unit_price REAL NOT NULL DEFAULT 0.0,
    line_total REAL NOT NULL DEFAULT 0.0,
    FOREIGN KEY (order_id) REFERENCES Orders(order_id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (product_id) REFERENCES Products(product_id) ON UPDATE CASCADE ON DELETE SET NULL
);

-- ==========================================================
-- 9. ตารางรายการคืนสินค้า (Return Items / RB)
--    ใช้ตรวจสอบว่าจำนวนที่คืนสะสมไม่เกินจำนวนที่ขายในบิลเดิม
-- ==========================================================
CREATE TABLE IF NOT EXISTS Return_Items (
    return_item_id TEXT PRIMARY KEY,
    return_order_id TEXT NOT NULL,
    original_order_id TEXT NOT NULL,
    product_id TEXT,
    product_name TEXT NOT NULL,
    quantity INTEGER NOT NULL CHECK(quantity > 0),
    reason TEXT,
    return_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (return_order_id) REFERENCES Orders(order_id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (product_id) REFERENCES Products(product_id) ON UPDATE CASCADE ON DELETE SET NULL
);

-- ==========================================================
-- 10. ดัชนีเพื่อเพิ่มประสิทธิภาพการค้นหา (Indexes)
-- ==========================================================
CREATE INDEX IF NOT EXISTS idx_products_name ON Products(name);
CREATE INDEX IF NOT EXISTS idx_products_category_id ON Products(category_id);
CREATE INDEX IF NOT EXISTS idx_products_supplier_id ON Products(supplier_id);
CREATE INDEX IF NOT EXISTS idx_stock_transactions_product_id ON Stock_Transactions(product_id);
CREATE INDEX IF NOT EXISTS idx_stock_transactions_date ON Stock_Transactions(transaction_date);
CREATE INDEX IF NOT EXISTS idx_orders_status ON Orders(status);
CREATE INDEX IF NOT EXISTS idx_orders_date ON Orders(order_date);
CREATE INDEX IF NOT EXISTS idx_orders_original_order_id ON Orders(original_order_id);
CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON Order_Items(order_id);
CREATE INDEX IF NOT EXISTS idx_order_items_product_id ON Order_Items(product_id);
CREATE INDEX IF NOT EXISTS idx_return_items_original_order_id ON Return_Items(original_order_id);
