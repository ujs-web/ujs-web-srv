# Database Bridge (db_bridge)

`db_bridge` 模块负责处理与 PostgreSQL 数据库的交互。它支持基于 `diesel` 的连接池管理，并提供了灵活的动态 SQL 操作能力，能够处理任意表结构的增删改查。

## 1. 核心功能

### 1.1 连接管理
- **连接池**：使用 `r2d2` 管理 `PgConnection` 连接池。
- **配置**：支持从 `.env` 文件或环境变量 `DATABASE_URL` 加载数据库地址。默认连接：`postgres://ever@localhost/postgres`。

### 1.2 动态 SQL 操作 (`ops.rs`)
模块提供了绕过编译期 Schema 检查的动态操作接口：

- **`dynamic_insert`**: 支持向指定表名插入多列数据。内部使用 `sql_query` 配合运行时绑定。
- **`dynamic_query`**: 使用 `diesel-dynamic-schema` 实现对任意表的查询。支持选择性列查询，并将结果映射为 `Vec<Vec<String>>`。
- **`dynamic_delete`**: 根据指定的列和值删除记录。
- **测试辅助**: 提供 `setup_test_table` 和 `drop_test_table` 用于快速创建/销毁测试环境。

## 2. Rust 使用示例

```rust
use crate::db_bridge::establish_connection_pool;
use crate::db_bridge::ops::*;

let pool = establish_connection_pool();
let mut conn = pool.get().unwrap();

// 插入数据
let columns = vec![
    ("name", "Alice"),
    ("email", "alice@example.com"),
];
dynamic_insert(&mut conn, "users", columns).expect("Insert failed");

// 查询数据
let results = dynamic_query(&mut conn, "users", vec!["name", "email"]).expect("Query failed");
for row in results {
    println!("Name: {}, Email: {}", row[0], row[1]);
}
```

## 3. JavaScript 集成 (via `js_bridge`)

`db_bridge` 的功能通过 `js_bridge` 暴露给 JavaScript 运行时。

### 3.1 注入对象 `db`
在 JS 环境中可以直接使用异步的 `db` 对象：

- **`await db.execute(sql)`**: 执行 DML 语句（INSERT, UPDATE, DELETE, CREATE, DROP），返回受影响的行数。
- **`await db.query(sql)`**: 执行查询语句。
    - **自动类型转换**：PostgreSQL 的 `Integer` 自动转换为 JS `Number`，`Text` 转换为 `String`。
    - **动态列名**：返回对象数组，Key 为数据库列名。

### 3.2 JS 使用示例

```javascript
// 在 scripts/*.js 中编写
async function main() {
    // 创建表
    await db.execute("CREATE TABLE IF NOT EXISTS test (id SERIAL, val TEXT)");
    
    // 插入并获取结果
    await db.execute("INSERT INTO test (val) VALUES ('hello')");
    
    // 查询
    const rows = await db.query("SELECT * FROM test");
    console.log(rows[0].id);  // 输出数字类型
    console.log(rows[0].val); // 输出 "hello"
}
```

## 4. 测试

模块包含了完善的集成测试，验证了从 Rust 直接调用和从 JS 脚本间接调用的正确性。

### 运行 Rust 单元测试
```bash
cargo test db_bridge
```

### 运行 JS 桥接测试
```bash
cargo test js_bridge
```

## 5. 依赖项
- `diesel`: 核心 ORM 和 SQL 执行器。
- `diesel-dynamic-schema`: 提供运行时动态表/列定义支持。
- `r2d2`: 生产级连接池。
- `serde_json`: 用于 JS 结果集的序列化与类型映射。
