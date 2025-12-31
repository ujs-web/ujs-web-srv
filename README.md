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

---

## 2. 架构与实现 (Architecture & Implementation)

### 2.1 核心技术栈
*   **后端框架**：Axum (基于 Tokio)
*   **JS 引擎**：deno_core (基于 V8)
*   **转译引擎**：deno_ast (用于 TS 到 JS 的转译)
*   **序列化**：serde/serde_json

### 2.2 系统架构设计

#### 2.2.1 隔离与并发
由于 `deno_core::JsRuntime` 是非 `Send` 的，系统采用了 **Thread-per-Request** 模型：
1.  Axum 接收到请求后，启动一个独立的 `std::thread`。
2.  在工作线程内部创建一个新的 `JsRuntime` 实例。
3.  通过 `oneshot` channel 实现同步/异步桥接，将 JS 执行结果传回 Axum 主运行环境。

#### 2.2.2 模块加载机制 (TsModuleLoader)
实现自定义 `ModuleLoader`：
*   **路径解析**：支持相对路径导入。
*   **自动转译**：在加载文件时，根据文件后缀（`.ts`, `.tsx`, `.mts` 等）利用 `deno_ast` 进行实时转译。
*   **代码注入**：在脚本执行前，通过 `execute_script` 将 `globalThis.request` 对象注入全局作用域。

#### 2.2.3 扩展插件 (Extensions)
通过 `deno_core::extension!` 定义了 `web_runtime` 扩展，暴露以下 Ops 给 JS：
*   `op_log`: 将信息打印到 Rust 控制台。
*   `op_delay`: 异步延时函数。
*   `op_send_response`: 将构造好的响应对象提交回 Rust 端。

---

## 3. 使用指南 (Usage Guide)

### 3.1 目录结构
*   `src/main.rs`: Rust 核心实现。
*   `scripts/`: 业务脚本存放目录，支持 `.js` 和 `.ts`。

### 3.2 脚本编写示例 (TypeScript)
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

---

## 4. 验证结果 (Verification)

目前已通过以下场景验证：
1.  **基础请求**：成功处理 POST 请求并读取 Body。
2.  **异步测试**：`await op_delay` 正常工作，不阻塞其他请求。
3.  **ESM 综合测试**：支持 Class 定义及跨文件 `import`。
4.  **TypeScript 测试**：自动转译带类型注解和接口的 `.ts` 文件。

可以通过以下命令启动：
```bash
cargo run
```
访问示例：`curl http://localhost:3001/js/test_ts.ts`
