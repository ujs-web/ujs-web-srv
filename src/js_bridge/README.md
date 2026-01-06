# JavaScript Bridge (js_bridge)

`js_bridge` 模块是整个框架的核心，负责在 Rust 的 Axum Web 框架和 JavaScript 运行时之间建立桥梁，使开发者能够使用 JavaScript/TypeScript 编写 Web 应用的业务逻辑。

## 1. 模块架构

### 1.1 组件概览

```
js_bridge/
├── mod.rs           # 模块导出和测试
├── handler.rs       # HTTP 请求处理器
├── executor.rs      # 脚本执行器
├── loader.rs        # 模块加载器
├── models.rs        # 数据模型定义
├── ops.rs           # Rust Ops 定义
└── init.js          # JavaScript 运行时初始化脚本
```

### 1.2 请求处理流程

```
HTTP Request
    ↓
Axum Router (/js/{*script_path})
    ↓
handle_js_script (handler.rs)
    ↓
提取请求信息 (method, path, headers, body)
    ↓
构建 JsRequest 对象
    ↓
ScriptExecutor::execute (executor.rs)
    ↓
rayon::spawn (独立线程)
    ↓
创建 JsRuntime 实例
    ↓
加载并执行脚本
    ↓
JavaScript 调用 op_send_response
    ↓
通过 oneshot channel 返回结果
    ↓
JsResponse → HTTP Response
```

## 2. 核心组件详解

### 2.1 Handler (handler.rs)

**职责**：接收 Axum 的 HTTP 请求，提取元数据，构建运行时配置。

**关键函数**：
```rust
pub async fn handle_js_script(
    State(pool): State<DbPool>,
    Path(script_name): Path<String>,
    req: Request,
) -> impl IntoResponse
```

**处理步骤**：
1. 从请求中提取 HTTP 方法、路径、请求头和请求体
2. 将请求头转换为 `HashMap<String, String>`
3. 读取请求体（最大 1MB）
4. 构建 `JsRequest` 对象
5. 创建 `RuntimeConfig` 配置
6. 调用 `ScriptExecutor::execute` 执行脚本

### 2.2 Executor (executor.rs)

**职责**：管理 JavaScript 运行时的生命周期，执行脚本并返回结果。

**核心结构**：
```rust
pub struct RuntimeConfig {
    pub script_path: String,
    pub request: JsRequest,
    pub db_pool: DbPool,
}
```

**执行流程**：
1. 使用 `rayon::spawn` 在独立线程中执行
2. 创建新的 `JsRuntime` 实例
3. 注入 `web_runtime` 扩展和 `TsModuleLoader`
4. 将 `JsRequest` 添加到资源表
5. 注入 `globalThis.__JS_REQUEST_RID__`
6. 注入数据库连接池
7. 加载并执行 ES 模块
8. 运行事件循环
9. 通过 `oneshot::channel` 等待响应

**线程隔离**：
- 每个 HTTP 请求在独立的线程中执行
- 避免了 `JsRuntime` 的线程安全问题
- 使用 `rayon` 线程池提高性能

### 2.3 Loader (loader.rs)

**职责**：实现自定义模块加载器，支持 TypeScript 转译。

**核心功能**：
1. **路径解析**：将相对路径转换为绝对路径
2. **媒体类型检测**：根据文件扩展名识别文件类型
3. **TypeScript 转译**：使用 `deno_ast` 将 TS 转换为 JS
4. **模块源码返回**：返回可执行的 JavaScript 代码

**支持的文件类型**：
- `.js` - JavaScript
- `.ts` - TypeScript
- `.mts` - ES Module TypeScript
- `.cts` - CommonJS TypeScript
- `.jsx` - JavaScript JSX
- `.tsx` - TypeScript JSX

**转译配置**：
```rust
let transpiled = parsed.transpile(
    &deno_ast::TranspileOptions { ..Default::default() },
    &deno_ast::TranspileModuleOptions::default(),
    &deno_ast::EmitOptions::default(),
)
```

### 2.4 Models (models.rs)

**职责**：定义 JavaScript 和 Rust 之间的数据模型。

#### JsRequest
表示 HTTP 请求，实现 `Resource` trait 以便在 V8 中使用。

```rust
pub struct JsRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}
```

**方法**：
- `get_method()` - 获取 HTTP 方法
- `get_path()` - 获取请求路径
- `get_headers()` - 获取所有请求头
- `get_body()` - 获取请求体
- `get_header(key)` - 获取指定请求头

#### JsResponse
表示 HTTP 响应，实现 `IntoResponse` trait 以便转换为 Axum 响应。

```rust
pub struct JsResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}
```

**构造方法**：
- `new(status, body)` - 创建新响应
- `internal_error(msg)` - 创建 500 错误响应
- `not_found(msg)` - 创建 404 响应

### 2.5 Ops (ops.rs)

**职责**：定义 Rust 到 JavaScript 的操作接口。

#### 请求操作 Ops

**op_req_method**
```rust
#[op2]
#[string]
pub fn op_req_method(state: &mut OpState, #[smi] rid: u32) -> String
```
获取请求的 HTTP 方法。

**op_req_path**
```rust
#[op2]
#[string]
pub fn op_req_path(state: &mut OpState, #[smi] rid: u32) -> String
```
获取请求的路径。

**op_req_headers**
```rust
#[op2]
#[serde]
pub fn op_req_headers(state: &mut OpState, #[smi] rid: u32) -> HashMap<String, String>
```
获取所有请求头。

**op_req_body**
```rust
#[op2]
#[string]
pub fn op_req_body(state: &mut OpState, #[smi] rid: u32) -> String
```
获取请求体。

**op_req_get_header**
```rust
#[op2]
#[string]
pub fn op_req_get_header(
    state: &mut OpState,
    #[smi] rid: u32,
    #[string] key: String,
) -> Option<String>
```
获取指定名称的请求头。

**op_req_close**
```rust
#[op2(fast)]
pub fn op_req_close(state: &mut OpState, #[smi] rid: u32)
```
关闭请求资源。

#### 响应操作 Ops

**op_send_response**
```rust
#[op2]
pub fn op_send_response(state: &mut OpState, #[serde] res: JsResponse)
```
发送响应回 Rust 端，通过 `oneshot::channel` 传递。

#### 工具 Ops

**op_log**
```rust
#[op2(fast)]
pub fn op_log(#[string] msg: String)
```
将消息打印到 Rust 控制台。

**op_delay**
```rust
#[op2(async)]
pub async fn op_delay(ms: u32)
```
异步延时函数，支持 `await`。

#### 数据库 Ops

**op_sql_execute**
```rust
#[op2(fast)]
pub fn op_sql_execute(state: &mut OpState, #[string] sql: String) -> u32
```
执行 DML 语句，返回受影响的行数。

**op_sql_query**
```rust
#[op2]
#[serde]
pub fn op_sql_query(state: &mut OpState, #[string] sql: String) -> serde_json::Value
```
执行查询语句，返回 JSON 格式的结果。

### 2.6 初始化脚本 (init.js)

**职责**：在 JavaScript 运行时初始化全局对象和 API。

#### Request 类
封装请求操作，提供面向对象的 API：

```javascript
export class Request {
    #rid;

    constructor() {
        this.#rid = globalThis.__JS_REQUEST_RID__;
    }

    method() { return op_req_method(this.#rid) }
    path() { return op_req_path(this.#rid) }
    headers() { return op_req_headers(this.#rid) }
    body() { return op_req_body(this.#rid) }
    header(k) { return op_req_get_header(this.#rid, k) }
    close() { return op_req_close(this.#rid) }
}
```

#### 全局对象

**globalThis.request**
```javascript
globalThis.request = new Request();
```
提供对当前 HTTP 请求的访问。

**globalThis.db**
```javascript
globalThis.db = {
    execute: (sql) => op_sql_execute(sql),
    query: (sql) => op_sql_query(sql),
};
```
提供数据库操作接口。

## 3. JavaScript API 参考

### 3.1 请求对象 (globalThis.request)

#### request.method()
获取 HTTP 请求方法。

**返回值**：字符串（如 "GET", "POST", "PUT", "DELETE"）

**示例**：
```javascript
const method = globalThis.request.method();
console.log(method); // "POST"
```

#### request.path()
获取请求路径。

**返回值**：字符串（如 "/js/hello.js"）

**示例**：
```javascript
const path = globalThis.request.path();
console.log(path); // "/js/hello.js"
```

#### request.headers()
获取所有请求头。

**返回值**：对象（键值对）

**示例**：
```javascript
const headers = globalThis.request.headers();
console.log(headers['content-type']); // "application/json"
```

#### request.body()
获取请求体。

**返回值**：字符串

**示例**：
```javascript
const body = globalThis.request.body();
const data = JSON.parse(body);
console.log(data.name); // "Alice"
```

#### request.header(key)
获取指定请求头。

**参数**：
- `key` (string): 请求头名称

**返回值**：字符串或 undefined

**示例**：
```javascript
const contentType = globalThis.request.header('content-type');
console.log(contentType); // "application/json"
```

### 3.2 数据库对象 (globalThis.db)

#### db.execute(sql)
执行 DML 语句。

**参数**：
- `sql` (string): SQL 语句

**返回值**：受影响的行数（number）

**示例**：
```javascript
const rows = await db.execute("INSERT INTO users (name) VALUES ('Alice')");
console.log(rows); // 1
```

#### db.query(sql)
执行查询语句。

**参数**：
- `sql` (string): SQL 查询语句

**返回值**：对象数组（Array<Object>）

**示例**：
```javascript
const users = await db.query("SELECT * FROM users");
console.log(users[0].name); // "Alice"
```

### 3.3 工具函数

#### Deno.core.ops.op_log(msg)
输出日志到控制台。

**参数**：
- `msg` (string): 日志消息

**示例**：
```javascript
Deno.core.ops.op_log("Processing request...");
```

#### Deno.core.ops.op_delay(ms)
异步延时。

**参数**：
- `ms` (number): 延时毫秒数

**示例**：
```javascript
await Deno.core.ops.op_delay(1000); // 延时 1 秒
```

### 3.4 响应发送

#### Deno.core.ops.op_send_response(response)
发送 HTTP 响应。

**参数**：
- `response` (object): 响应对象
  - `status` (number): HTTP 状态码
  - `headers` (object): 响应头（可选）
  - `body` (string): 响应体

**示例**：
```javascript
Deno.core.ops.op_send_response({
    status: 200,
    headers: {
        "Content-Type": "application/json"
    },
    body: JSON.stringify({ message: "Hello" })
});
```

## 4. 测试

模块包含完整的测试套件，位于 [mod.rs](mod.rs) 中。

### 4.1 单元测试

**test_request_getters**
测试 `JsRequest` 的 getter 方法。

**test_js_response_into_response**
测试 `JsResponse` 到 Axum 响应的转换。

### 4.2 集成测试

**test_js_request_methods**
测试完整的 HTTP 请求处理流程。

**test_js_async_with_threadpool**
测试异步操作和线程池。

**test_script_not_found**
测试脚本不存在时的错误处理。

**test_ts_transpilation**
测试 TypeScript 转译功能。

**test_js_syntax_error_handling**
测试语法错误的处理。

**test_op_req_ops_directly**
测试 Ops 的直接调用。

**test_js_sql_operations**
测试数据库操作的集成。

**test_js_sql_dynamic_row**
测试动态行查询功能。

### 4.3 运行测试

```bash
# 运行所有测试
cargo test

# 运行 js_bridge 模块测试
cargo test js_bridge

# 运行特定测试
cargo test test_js_request_methods

# 显示测试输出
cargo test -- --nocapture
```

## 5. 最佳实践

### 5.1 错误处理

在 JavaScript 脚本中使用 try-catch 捕获错误：

```javascript
async function handleRequest() {
    try {
        const data = await db.query("SELECT * FROM users");
        Deno.core.ops.op_send_response({
            status: 200,
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(data)
        });
    } catch (error) {
        Deno.core.ops.op_log(`Error: ${error.message}`);
        Deno.core.ops.op_send_response({
            status: 500,
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ error: error.message })
        });
    }
}

handleRequest();
```

### 5.2 异步操作

使用 `async/await` 处理异步操作：

```javascript
async function handleRequest() {
    // 异步延时
    await Deno.core.ops.op_delay(100);

    // 异步数据库查询
    const users = await db.query("SELECT * FROM users");

    // 返回响应
    Deno.core.ops.op_send_response({
        status: 200,
        body: JSON.stringify(users)
    });
}

handleRequest();
```

### 5.3 模块化

使用 ES 模块组织代码：

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
        body: JSON.stringify({ name, timestamp })
    });
}

handleRequest();
```

### 5.4 TypeScript 使用

利用 TypeScript 的类型系统：

```typescript
// scripts/types.ts
interface User {
    id: number;
    name: string;
    email: string;
}

interface ApiResponse {
    success: boolean;
    data?: User[];
    error?: string;
}
```

```typescript
// scripts/main.ts
import type { User, ApiResponse } from './types.ts';

async function handleRequest() {
    try {
        const users: User[] = await db.query("SELECT * FROM users");

        const response: ApiResponse = {
            success: true,
            data: users
        };

        Deno.core.ops.op_send_response({
            status: 200,
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(response)
        });
    } catch (error) {
        const errorResponse: ApiResponse = {
            success: false,
            error: error.message
        };

        Deno.core.ops.op_send_response({
            status: 500,
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(errorResponse)
        });
    }
}

handleRequest();
```

## 6. 性能优化

### 6.1 减少内存分配

- 重用对象而不是频繁创建新对象
- 使用字符串模板而不是字符串拼接

### 6.2 异步操作优化

- 避免在循环中使用 `await`，改用 `Promise.all`
- 合理使用 `op_delay` 避免阻塞

### 6.3 数据库查询优化

- 使用索引提高查询性能
- 只查询需要的列
- 使用批量操作代替单条操作

## 7. 调试技巧

### 7.1 日志输出

使用 `op_log` 输出调试信息：

```javascript
Deno.core.ops.op_log("Starting request processing...");
Deno.core.ops.op_log(`Request method: ${globalThis.request.method()}`);
```

### 7.2 错误追踪

捕获并输出完整的错误信息：

```javascript
try {
    // 代码
} catch (error) {
    Deno.core.ops.op_log(`Error: ${error.message}`);
    Deno.core.ops.op_log(`Stack: ${error.stack}`);
}
```

### 7.3 Rust 端调试

在 Rust 代码中使用 `eprintln!` 输出调试信息：

```rust
eprintln!("Script path: {}", script_path);
eprintln!("Request: {:?}", request);
```

## 8. 扩展开发

### 8.1 添加新的 Op

在 [ops.rs](ops.rs) 中添加新的 Op：

```rust
#[op2]
pub fn op_custom_operation(state: &mut OpState, #[string] param: String) -> String {
    format!("Processed: {}", param)
}
```

在 `extension!` 宏中注册：

```rust
extension!(
    web_runtime,
    ops = [
        // ... 其他 ops
        op_custom_operation
    ],
    esm_entry_point = "ext:web_runtime/init.js",
    esm = [ dir "src/js_bridge", "init.js" ],
);
```

在 [init.js](init.js) 中导出：

```javascript
import { op_custom_operation } from 'ext:core/ops';

globalThis.custom = {
    process: (param) => op_custom_operation(param)
};
```

### 8.2 添加新的全局对象

在 [init.js](init.js) 中添加：

```javascript
globalThis.myHelper = {
    formatDate: (date) => date.toISOString(),
    sanitize: (input) => input.trim().toLowerCase()
};
```

在脚本中使用：

```javascript
const formatted = globalThis.myHelper.formatDate(new Date());
```

## 9. 相关文档

- [主 README](../../README.md) - 项目总览
- [Database Bridge](../db_bridge/README.md) - 数据库桥接模块文档
- [Deno Core 文档](https://docs.rs/deno_core/) - Deno Core API 参考
- [Axum 文档](https://docs.rs/axum/) - Axum Web 框架文档
