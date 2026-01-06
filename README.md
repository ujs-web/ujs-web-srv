# Axum + DenoCore Web 开发框架文档

本项目是一个基于 Rust 的 Web 开发框架，结合了 `axum` 高性能 Web 框架和 `deno_core` 的 JavaScript 运行时，旨在提供一个可以通过 JS/TS 编写业务逻辑的动态 Web 服务。

## 1. 需求说明 (Requirements)

### 1.1 基础功能映射
*   **请求映射**：将 HTTP 请求的元数据（Method, Path, Headers, Body）无缝映射到 JavaScript 运行环境中，使脚本能够读取当前请求信息。
*   **响应控制**：脚本能够通过 JS 修改响应头（Headers）、状态码（Status Code）并输出响应体（Body）。

### 1.2 运行时能力
*   **异步支持**：运行时必须支持 `async/await` 语法，允许在 JS 中执行非阻塞操作（如延时、IO 等）。
*   **模块化支持**：
    *   **ES 模块 (ESM)**：支持 `import/export` 语法及 Top-level `await`。
    *   **TypeScript (TS)**：原生支持加载并执行 `.ts` 文件，自动进行转译。
*   **线程安全**：`JsRuntime` 应能在多线程环境下高效工作，每个请求独立分配运行时上下文。

### 1.3 动态路由
*   **脚本路由**：提供一个 API 端点（如 `/js/{*path}`），根据 URL 路径动态匹配并加载对应的本地 JS/TS 脚本文件执行。

### 1.4 数据库集成
*   **PostgreSQL 支持**：内置 PostgreSQL 数据库连接池，支持在 JS 中直接执行 SQL 查询。
*   **动态 SQL**：无需预定义 Schema，支持任意表的增删改查操作。
*   **类型映射**：自动将数据库类型映射到 JavaScript 类型（Integer → Number, Text → String 等）。

---

## 2. 架构与实现 (Architecture & Implementation)

### 2.1 核心技术栈
*   **后端框架**：Axum (基于 Tokio)
*   **JS 引擎**：deno_core (基于 V8)
*   **转译引擎**：deno_ast (用于 TS 到 JS 的转译)
*   **序列化**：serde/serde_json
*   **数据库**：Diesel ORM + PostgreSQL
*   **连接池**：r2d2
*   **并发**：rayon (线程池)

### 2.2 系统架构设计

#### 2.2.1 隔离与并发
由于 `deno_core::JsRuntime` 是非 `Send` 的，系统采用了 **Thread-per-Request** 模型：
1.  Axum 接收到请求后，使用 `rayon::spawn` 启动一个独立的工作线程。
2.  在工作线程内部创建一个新的 `JsRuntime` 实例。
3.  通过 `oneshot` channel 实现同步/异步桥接，将 JS 执行结果传回 Axum 主运行环境。

#### 2.2.2 模块加载机制 (TsModuleLoader)
实现自定义 `TsModuleLoader`：
*   **路径解析**：支持相对路径导入。
*   **自动转译**：在加载文件时，根据文件后缀（`.ts`, `.tsx`, `.mts` 等）利用 `deno_ast` 进行实时转译。
*   **代码注入**：在脚本执行前，通过 `execute_script` 将 `globalThis.request` 对象注入全局作用域。

#### 2.2.3 扩展插件 (Extensions)
通过 `deno_core::extension!` 定义了 `web_runtime` 扩展，暴露以下 Ops 给 JS：

**请求操作**：
*   `op_req_method`: 获取请求方法
*   `op_req_path`: 获取请求路径
*   `op_req_headers`: 获取请求头
*   `op_req_body`: 获取请求体
*   `op_req_get_header`: 获取指定请求头
*   `op_req_close`: 关闭请求资源

**响应操作**：
*   `op_send_response`: 将构造好的响应对象提交回 Rust 端

**工具函数**：
*   `op_log`: 将信息打印到 Rust 控制台
*   `op_delay`: 异步延时函数

**数据库操作**：
*   `op_sql_execute`: 执行 DML 语句（INSERT, UPDATE, DELETE, CREATE, DROP）
*   `op_sql_query`: 执行查询语句，返回 JSON 格式的结果

### 2.3 数据库桥接 (db_bridge)
`db_bridge` 模块提供 PostgreSQL 数据库的完整支持：

*   **连接池管理**：使用 `r2d2` 管理数据库连接池，支持高并发访问
*   **动态 Schema**：通过 `diesel-dynamic-schema` 实现运行时动态表/列定义
*   **类型转换**：自动将 PostgreSQL 类型映射到 JavaScript/JSON 类型
*   **SQL 注入防护**：使用参数化查询，防止 SQL 注入攻击

---

## 3. 使用指南 (Usage Guide)

### 3.1 目录结构
```
web_deno/
├── src/
│   ├── main.rs              # Rust 主程序入口
│   ├── js_bridge/           # JavaScript 桥接模块
│   │   ├── mod.rs           # 模块定义
│   │   ├── executor.rs      # 脚本执行器
│   │   ├── handler.rs       # HTTP 请求处理器
│   │   ├── loader.rs        # 模块加载器
│   │   ├── models.rs        # 数据模型
│   │   └── ops.rs           # Ops 定义
│   └── db_bridge/           # 数据库桥接模块
│       ├── mod.rs           # 模块定义
│       └── ops.rs           # 数据库操作
├── scripts/                 # 业务脚本存放目录
│   ├── hello.js             # 基础示例
│   ├── db_test.js           # 数据库测试
│   └── es_comprehensive.js  # ESM 综合测试
├── Cargo.toml               # Rust 依赖配置
└── .env                     # 环境变量配置（可选）
```

### 3.2 环境配置

创建 `.env` 文件配置数据库连接：
```env
DATABASE_URL=postgres://username:password@localhost/database_name
```

如果不配置，默认使用：`postgres://ever@localhost/postgres`

### 3.3 脚本编写示例

#### 3.3.1 基础 HTTP 请求处理 (TypeScript)
```typescript
// scripts/hello.ts
interface ResponseData {
  message: string;
  path: string;
}

const data: ResponseData = {
  message: "Hello from TS",
  path: (globalThis as any).request.path
};

// 发送响应
(Deno as any).core.ops.op_send_response({
  status: 200,
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify(data)
});
```

#### 3.3.2 数据库操作示例
```javascript
// scripts/db_example.js
async function handleRequest() {
  try {
    // 创建表
    await db.execute(`
      CREATE TABLE IF NOT EXISTS users (
        id SERIAL PRIMARY KEY,
        name TEXT NOT NULL,
        email TEXT NOT NULL
      )
    `);

    // 插入数据
    await db.execute(`
      INSERT INTO users (name, email) 
      VALUES ('Alice', 'alice@example.com')
    `);

    // 查询数据
    const users = await db.query(`
      SELECT * FROM users WHERE name = 'Alice'
    `);

    // 返回结果
    Deno.core.ops.op_send_response({
      status: 200,
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        success: true,
        users: users
      })
    });
  } catch (error) {
    Deno.core.ops.op_send_response({
      status: 500,
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ error: error.message })
    });
  }
}

handleRequest();
```

#### 3.3.3 异步操作示例
```javascript
// scripts/async_example.js
async function handleRequest() {
  // 模拟异步操作
  await Deno.core.ops.op_delay(100);

  // 日志输出
  Deno.core.ops.op_log("Processing request...");

  // 获取请求信息
  const method = globalThis.request.method;
  const path = globalThis.request.path;

  Deno.core.ops.op_send_response({
    status: 200,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      method,
      path,
      message: "Async operation completed"
    })
  });
}

handleRequest();
```

#### 3.3.4 ESM 模块示例
```javascript
// scripts/utils.js
export function formatDate(date) {
  return date.toISOString();
}

export function sanitizeInput(input) {
  return input.trim().toLowerCase();
}
```

```javascript
// scripts/main.js
import { formatDate, sanitizeInput } from './utils.js';

async function handleRequest() {
  const name = sanitizeInput(globalThis.request.body);
  const timestamp = formatDate(new Date());

  Deno.core.ops.op_send_response({
    status: 200,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      name,
      timestamp
    })
  });
}

handleRequest();
```

### 3.4 数据库 API 参考

#### db.execute(sql)
执行 DML 语句（INSERT, UPDATE, DELETE, CREATE, DROP）

**参数**：
- `sql` (string): SQL 语句

**返回值**：受影响的行数 (number)

**示例**：
```javascript
const affectedRows = await db.execute(
  "INSERT INTO users (name, email) VALUES ($1, $2)",
  ["John", "john@example.com"]
);
```

#### db.query(sql)
执行查询语句

**参数**：
- `sql` (string): SQL 查询语句

**返回值**：对象数组 (Array<Object>)

**类型映射**：
- PostgreSQL `INTEGER` → JavaScript `Number`
- PostgreSQL `BIGINT` → JavaScript `Number`
- PostgreSQL `TEXT` → JavaScript `String`
- PostgreSQL `BOOLEAN` → JavaScript `Boolean`
- PostgreSQL `DOUBLE` → JavaScript `Number`
- `NULL` → `null`

**示例**：
```javascript
const users = await db.query("SELECT id, name, email FROM users");
console.log(users[0].id);    // 数字类型
console.log(users[0].name);  // 字符串类型
```

---

## 4. 验证结果 (Verification)

目前已通过以下场景验证：

### 4.1 基础功能测试
1.  **基础请求**：成功处理 POST 请求并读取 Body。
2.  **异步测试**：`await op_delay` 正常工作，不阻塞其他请求。
3.  **ESM 综合测试**：支持 Class 定义及跨文件 `import`。
4.  **TypeScript 测试**：自动转译带类型注解和接口的 `.ts` 文件。

### 4.2 数据库功能测试
1.  **CRUD 操作**：完整的创建、读取、更新、删除操作验证
2.  **动态查询**：支持任意表结构的查询，无需预定义 Schema
3.  **类型映射**：验证 PostgreSQL 类型到 JavaScript 类型的正确转换
4.  **并发访问**：连接池在高并发场景下的稳定性测试

### 4.3 测试覆盖
项目包含完整的单元测试和集成测试：

**Rust 单元测试**：
- 数据库操作测试 (`db_bridge::tests`)
- 请求/响应模型测试

**集成测试**：
- JavaScript 脚本执行测试
- 数据库桥接测试
- 错误处理测试

运行测试：
```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test db_bridge
cargo test js_bridge
```

## 5. 快速开始 (Quick Start)

### 5.1 前置要求
- Rust 1.70+
- PostgreSQL 数据库（可选，如需使用数据库功能）

### 5.2 启动服务
```bash
# 克隆项目
git clone <repository-url>
cd web_deno

# 配置数据库（可选）
echo "DATABASE_URL=postgres://your_user:your_password@localhost/your_db" > .env

# 运行服务
cargo run
```

服务将在 `http://localhost:3001` 启动。

### 5.3 测试示例
```bash
# 基础请求测试
curl http://localhost:3001/js/hello.js

# 数据库测试
curl http://localhost:3001/js/db_test.js

# ESM 测试
curl http://localhost:3001/js/es_comprehensive.js

# TypeScript 测试
curl http://localhost:3001/js/test_ts.ts

# 自定义请求
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"name":"test"}' \
  http://localhost:3001/js/your_script.js
```

## 6. 开发指南 (Development Guide)

### 6.1 添加新的 Ops
在 [src/js_bridge/ops.rs](src/js_bridge/ops.rs) 中添加新的 Op：

```rust
#[op2]
pub fn op_your_operation(state: &mut OpState, #[string] param: String) -> String {
    // 实现你的逻辑
    format!("Processed: {}", param)
}
```

然后在 `extension!` 宏中注册：

```rust
extension!(
    web_runtime,
    ops = [
        // ... 其他 ops
        op_your_operation
    ],
    // ...
);
```

### 6.2 性能优化建议
1.  **连接池配置**：根据并发需求调整 `r2d2` 连接池大小
2.  **脚本缓存**：频繁使用的脚本可以考虑添加缓存机制
3.  **线程池调优**：根据 CPU 核心数调整 `rayon` 线程池配置

### 6.3 调试技巧
1.  **日志输出**：使用 `Deno.core.ops.op_log()` 在 JS 中输出调试信息
2.  **错误处理**：在脚本中使用 try-catch 捕获并返回错误信息
3.  **Rust 日志**：使用 `eprintln!()` 在 Rust 代码中输出调试信息

## 7. 常见问题 (FAQ)

### Q1: 脚本执行超时怎么办？
A: 脚本执行时间过长会导致请求超时。建议：
- 将长时间运行的任务拆分为多个异步操作
- 使用 `await op_delay()` 避免阻塞事件循环
- 优化数据库查询，添加必要的索引

### Q2: 如何处理 SQL 注入？
A: 当前版本使用参数化查询，但仍需注意：
- 不要直接拼接用户输入到 SQL 语句中
- 对用户输入进行验证和清理
- 使用最小权限原则配置数据库用户

### Q3: TypeScript 类型检查如何工作？
A: TypeScript 文件在运行时会被转译为 JavaScript：
- 类型信息在转译后会被移除
- 建议在开发时使用 `tsc --noEmit` 进行类型检查
- 类型错误不会影响运行时执行

### Q4: 如何部署到生产环境？
A: 生产部署建议：
- 使用环境变量管理配置
- 配置反向代理（如 Nginx）
- 启用 HTTPS
- 配置日志收集和监控
- 设置合理的超时和重试策略
