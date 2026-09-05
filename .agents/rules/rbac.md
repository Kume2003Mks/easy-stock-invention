# Role-Based Access Control (RBAC) Architecture

## 1. Overview & Data Model
ระบบควบคุมสิทธิ์การเข้าถึงและการทำงาน (Access Control) ถูกออกแบบโดยอิงตามบทบาทหน้าที่ (Roles) ผ่านโครงสร้างฐานข้อมูลแบบ **Many-to-Many** โดยใช้ **UUID v7** เป็น Primary Key ทั้งระบบ เพื่อให้ผู้ใช้งาน 1 คนสามารถถือได้หลายบทบาทพร้อมกัน (เช่น พนักงานหน้าร้านที่เป็นทั้ง Cashier และมีสิทธิ์เป็น Manager ได้):

* **`users`**: เก็บข้อมูลผู้ใช้งาน (เช่น `id`, `username`, `password_hash`, `is_active`)
* **`roles`**: กำหนดระดับบทบาทหลักในระบบ (`admin`, `manager`, `cashier`)
* **`user_roles`**: ตารางเชื่อมกลาง (Bridge Table) จับคู่ความสัมพันธ์ระหว่าง `user_id` (FK) และ `role_id` (FK)
* **`activity_logs`**: ตารางบันทึก Audit Trail ระดับปฏิบัติการ (เก็บ `user_id`, `action`, `target_table`, `target_id`, `old_values`, `new_values` ในรูปแบบ JSONB) เพื่อตรวจสอบย้อนหลังว่าใครทำอะไร

---

## 2. Role & Permission Matrix
การแบ่งขอบเขตหน้าที่และความรับผิดชอบตามโมเดลระบบ POS:

| บทบาท (Role) | สิทธิ์และการทำงานในระบบ (Permissions & Scope) | การเข้าถึงเมนูและฟังก์ชันที่อนุญาต |
| :--- | :--- | :--- |
| **Admin** | ผู้ดูแลระบบสูงสุด ควบคุมการตั้งค่า ความปลอดภัย และสิทธิ์ของผู้ใช้ทั้งหมด | • จัดการผู้ใช้งาน (สร้าง/ระงับ/กำหนดสิทธิ์ User)<br>• ตั้งค่าระบบส่วนกลาง (System Settings)<br>• กำหนดหมวดหมู่สินค้า (`categories`) และหน่วยนับ (`units`) |
| **Manager** | ดูแลงานปฏิบัติการ การควบคุมสินค้าคงคลัง และการตรวจสอบยอดเงิน | • อนุมัติการยกเลิกบิลขาย (Void Order / Return)<br>• จัดการราคาสินค้า ปรับยอดคลังสินค้า (Stock Adjustment)<br>• ดูรายงานสรุปยอดขาย (Reports & Dashboard)<br>• ตรวจสอบประวัติการทำงานย้อนหลัง (Audit Trail Logs) |
| **Cashier** | ปฏิบัติการขายหน้าร้าน (Point of Sale) เป็นหลัก | • เปิด/ปิดกะการทำงาน (Shift Management)<br>• สแกนบาร์โค้ด ค้นหาสินค้า และคำนวณราคาสินค้า<br>• เปิดบิลขาย (Create Order), รับชำระเงิน และพิมพ์ใบเสร็จ<br>• ไม่มีสิทธิ์แก้ไขราคา ปรับสต็อกสินค้า หรือ Void บิลโดยพลการ |

---

## 3. Implementation in Clean Architecture (Tauri + Rust)
การบังคับใช้กฎของ RBAC ถูกร้อยเรียงข้ามเลเยอร์ของระบบตามแนวคิด Clean Architecture:

* **Interface Adapters (`adapters/commands.rs`):**
  * ทำหน้าที่เป็นประตูด่านหน้า (Entry Gate) ดักจับคำสั่ง `#[tauri::command]` ที่ถูกเรียกข้ามมาจาก Frontend IPC Bridge
  * สกัดข้อมูลตัวตนและบทบาท (Role Checking) ของผู้ใช้งานจาก Session/State หรือ Security Context ก่อนส่งต่อให้ Use Case หากผู้ใช้ไม่มีสิทธิ์ที่กำหนด คำสั่งจะถูกปฏิเสธทันทีพร้อมคืนรหัสข้อผิดพลาดเชิงโครงสร้าง เช่น `ERR_UNAUTHORIZED` หรือ `ERR_FORBIDDEN`
* **Use Cases Layer (`use_cases/`):**
  * กำกับ Workflow ทางธุรกิจตามเงื่อนไขสิทธิ์ (Business Rule Authorization)
  * เช่น ในกระบวนการ `VoidOrderUseCase` ต้องมีการตรวจสอบและส่งต่อ Identity ของผู้มีอำนาจระดับ `manager` หรือ `admin` เท่านั้น
* **Audit Trail Coupling (`activity_logs`):**
  * ทุกการกระทำที่ผ่านการตรวจสอบสิทธิ์และส่งผลต่อการเปลี่ยนแปลงข้อมูลสำคัญ (เช่น `UPDATE_STOCK`, `VOID_ORDER`, `CREATE_CATEGORY`) จะต้องบันทึก `user_id` และบทบาทของผู้สั่งการลงใน Log เสมอ เพื่อให้การควบคุมสิทธิ์สามารถสอบทานได้แบบไม่สามารถปฏิเสธความรับผิดชอบ (Non-repudiation)
