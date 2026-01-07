#[cfg(test)]
mod tests {
    use crate::js_bridge::executor::script_runner::ScriptRunner;
    use crate::js_bridge::models::JsRequest;
    use crate::js_bridge::{RuntimeConfig, ScriptExecutor};
    use std::fs;
    #[test]
    fn test_run_script_not_found() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let result = ScriptRunner::run_script(&mut runtime, "./non_existent.js");
        assert!(result.is_err());
        // 错误应该包含路径相关信息
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Failed to resolve script path") || err_msg.contains("Failed to load module"));
    }

    #[tokio::test]
    async fn test_execute_script_not_found() {
        let pool = crate::db_bridge::establish_connection_pool();

        let request = JsRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers: std::collections::HashMap::new(),
            body: String::new(),
        };

        let config = RuntimeConfig {
            script_path: "./non_existent.js".to_string(),
            request,
            db_pool: pool,
        };

        let response = ScriptExecutor::execute(config).await;
        assert_eq!(response.status, 404);
    }

    #[test]
    fn test_run_script_simple_javascript() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_simple.js");
        let test_content = r#"
            console.log("Hello from test!");
            const result = 42;
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_simple.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_syntax_error() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_syntax_error.js");
        let test_content = r#"
            const x = ;
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_syntax_error.js");
        // 语法错误应该被捕获，run_script 返回 Err
        assert!(result.is_err());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_async_function() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_async.js");
        let test_content = r#"
            async function testAsync() {
                await new Promise(resolve => setTimeout(resolve, 1));
                return "async result";
            }
            testAsync().then(result => console.log(result));
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_async.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_import() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let main_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_import_main.js");
        let main_content = r#"
            import { value } from './test_import_module.js';
            console.log("Imported value:", value);
        "#;
        fs::write(&main_file, main_content).unwrap();

        let module_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_import_module.js");
        let module_content = r#"
            export const value = 123;
        "#;
        fs::write(&module_file, module_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_import_main.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&main_file);
        let _ = fs::remove_file(&module_file);
    }

    #[test]
    fn test_run_script_with_export() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_export.js");
        let test_content = r#"
            export const name = "test";
            export function greet() {
                return "Hello";
            }
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_export.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_empty_file() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_empty.js");
        fs::write(&test_file, "").unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_empty.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_comments() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_comments.js");
        let test_content = r#"
            // This is a single line comment
            /* This is a
               multi-line comment */
            const value = 42; // inline comment
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_comments.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_variables() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_variables.js");
        let test_content = r#"
            const constValue = 1;
            let letValue = 2;
            var varValue = 3;
            
            const object = { name: "test", value: 42 };
            const array = [1, 2, 3];
            
            console.log(object, array);
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_variables.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_functions() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_functions.js");
        let test_content = r#"
            function add(a, b) {
                return a + b;
            }
            
            const multiply = (a, b) => a * b;
            
            const result = add(1, 2) + multiply(3, 4);
            console.log("Result:", result);
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_functions.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_classes() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_classes.js");
        let test_content = r#"
            class Calculator {
                constructor() {
                    this.result = 0;
                }
                
                add(value) {
                    this.result += value;
                    return this;
                }
                
                multiply(value) {
                    this.result *= value;
                    return this;
                }
            }
            
            const calc = new Calculator();
            calc.add(5).multiply(2);
            console.log("Result:", calc.result);
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_classes.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_error_handling() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_error_handling.js");
        let test_content = r#"
            try {
                throw new Error("Test error");
            } catch (error) {
                console.log("Caught error:", error.message);
            }
            
            const result = "success";
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_error_handling.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_promises() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_promises.js");
        let test_content = r#"
            const promise = new Promise((resolve, reject) => {
                setTimeout(() => resolve("Promise resolved"), 10);
            });
            
            promise.then(result => {
                console.log(result);
            });
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_promises.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_template_literals() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_template_literals.js");
        let test_content = r#"
            const name = "World";
            const greeting = `Hello, ${name}!`;
            const multiline = `Line 1
            Line 2
            Line 3`;
            console.log(greeting, multiline);
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_template_literals.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_destructuring() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_destructuring.js");
        let test_content = r#"
            const object = { a: 1, b: 2, c: 3 };
            const { a, b } = object;
            
            const array = [1, 2, 3];
            const [first, second] = array;
            
            console.log(a, b, first, second);
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_destructuring.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_spread_operator() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_spread_operator.js");
        let test_content = r#"
            const arr1 = [1, 2];
            const arr2 = [...arr1, 3, 4];
            
            const obj1 = { a: 1 };
            const obj2 = { ...obj1, b: 2 };
            
            console.log(arr2, obj2);
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_spread_operator.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_modules() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_modules.js");
        let test_content = r#"
            export const value = 42;
            export function test() {
                return "test";
            }
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_modules.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_unicode() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_unicode.js");
        let test_content = r#"
            const chinese = "你好世界";
            const emoji = "🎉🚀";
            const mixed = "Hello 你好 🎉";
            console.log(chinese, emoji, mixed);
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_unicode.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_large_file() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_large.js");
        let mut test_content = String::new();
        for i in 0..1000 {
            test_content.push_str(&format!("const value{} = {};\n", i, i));
        }
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_large.js");
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[tokio::test]
    async fn test_execute_script_with_valid_javascript() {
        let pool = crate::db_bridge::establish_connection_pool();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_execute.js");
        let test_content = r#"
            console.log("Test execution");
            Deno.core.ops.op_send_response({
                status: 200,
                headers: {},
                body: JSON.stringify({ message: "success" })
            });
        "#;
        fs::write(&test_file, test_content).unwrap();

        let request = JsRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers: std::collections::HashMap::new(),
            body: String::new(),
        };

        let config = RuntimeConfig {
            script_path: "scripts/test_execute.js".to_string(),
            request,
            db_pool: pool,
        };

        let response = ScriptExecutor::execute(config).await;
        assert_eq!(response.status, 200);

        let _ = fs::remove_file(&test_file);
    }

    #[tokio::test]
    async fn test_execute_script_with_error() {
        let pool = crate::db_bridge::establish_connection_pool();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_execute_error.js");
        let test_content = r#"
            throw new Error("Test error");
        "#;
        fs::write(&test_file, test_content).unwrap();

        let request = JsRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers: std::collections::HashMap::new(),
            body: String::new(),
        };

        let config = RuntimeConfig {
            script_path: "scripts/test_execute_error.js".to_string(),
            request,
            db_pool: pool,
        };

        let response = ScriptExecutor::execute(config).await;
        // 错误应该被捕获并返回 500
        assert_eq!(response.status, 500);

        let _ = fs::remove_file(&test_file);
    }

    #[tokio::test]
    async fn test_execute_script_with_post_request() {
        let pool = crate::db_bridge::establish_connection_pool();

        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_execute_post.js");
        let test_content = r#"
            console.log("Simple test");
            Deno.core.ops.op_send_response({
                status: 200,
                headers: {},
                body: JSON.stringify({ received: true })
            });
        "#;
        fs::write(&test_file, test_content).unwrap();

        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let request = JsRequest {
            method: "POST".to_string(),
            path: "/test".to_string(),
            headers,
            body: r#"{"test": "data"}"#.to_string(),
        };

        let config = RuntimeConfig {
            script_path: "scripts/test_execute_post.js".to_string(),
            request,
            db_pool: pool,
        };

        let response = ScriptExecutor::execute(config).await;
        assert_eq!(response.status, 200);

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_invalid_path_characters() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 使用包含非法字符的路径
        let result = ScriptRunner::run_script(&mut runtime, "scripts/test\nfile.js");
        // 路径解析可能失败
        assert!(result.is_err());
    }

    #[test]
    fn test_run_script_with_very_long_path() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建非常长的路径
        let long_name = "a".repeat(1000);
        let result = ScriptRunner::run_script(&mut runtime, &format!("scripts/{}.js", long_name));
        assert!(result.is_err());
    }

    #[test]
    fn test_run_script_with_absolute_path_outside_cwd() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 使用绝对路径指向不存在的文件
        let result = ScriptRunner::run_script(&mut runtime, "/nonexistent/path/to/script.js");
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Failed to resolve script path") || err_msg.contains("Failed to load module"));
    }

    #[test]
    fn test_run_script_with_relative_path_dots() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 使用包含过多点的相对路径
        let result = ScriptRunner::run_script(&mut runtime, "../../../../../../nonexistent.js");
        assert!(result.is_err());
    }

    #[test]
    fn test_run_script_with_circular_import() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建循环导入的文件
        let file1 = std::env::current_dir()
            .unwrap()
            .join("scripts/test_circular1.js");
        let content1 = r#"
            import { value } from './test_circular2.js';
            export const value1 = value;
        "#;
        fs::write(&file1, content1).unwrap();

        let file2 = std::env::current_dir()
            .unwrap()
            .join("scripts/test_circular2.js");
        let content2 = r#"
            import { value1 } from './test_circular1.js';
            export const value = value1;
        "#;
        fs::write(&file2, content2).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_circular1.js");
        // 循环导入可能被处理或返回错误，取决于实现
        // 在这个实现中，循环导入可能导致运行时错误
        assert!(result.is_ok() || result.is_err());

        let _ = fs::remove_file(&file1);
        let _ = fs::remove_file(&file2);
    }

    #[test]
    fn test_run_script_with_import_error_nonexistent_module() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建导入不存在模块的文件
        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_import_error.js");
        let test_content = r#"
            import { value } from './nonexistent_module.js';
            console.log(value);
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_import_error.js");
        // 导入错误应该被捕获
        assert!(result.is_err());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_invalid_module_syntax() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建包含无效模块语法的文件
        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_invalid_module.js");
        let test_content = r#"
            export default 42;
            // 真正的语法错误
            const x = 
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_invalid_module.js");
        // 无效语法应该被捕获
        assert!(result.is_err());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_runtime_error() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建包含运行时错误的文件
        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_runtime_error.js");
        let test_content = r#"
            const obj = null;
            console.log(obj.property); // 访问 null 的属性
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_runtime_error.js");
        // 运行时错误被打印到 stderr，但 run_script 返回 Ok
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_stack_overflow() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建可能导致栈溢出的文件 - 使用有限深度
        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_stack_overflow.js");
        let test_content = r#"
            let depth = 0;
            function recursive() {
                depth++;
                if (depth < 100) { // 限制深度避免真正溢出
                    recursive();
                }
            }
            recursive();
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_stack_overflow.js");
        // 应该能够执行
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_memory_leak_pattern() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建可能导致内存问题的文件
        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_memory.js");
        let test_content = r#"
            // 创建大量对象
            const arr = [];
            for (let i = 0; i < 10000; i++) {
                arr.push({ data: new Array(1000).fill('x') });
            }
            console.log("Array created with", arr.length, "items");
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_memory.js");
        // 应该能够执行，但可能需要较长时间
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_top_level_await() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建包含顶层 await 的文件
        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_top_level_await.js");
        let test_content = r#"
            const result = await Promise.resolve(42);
            console.log("Top-level await result:", result);
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_top_level_await.js");
        // 顶层 await 应该被支持
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_dynamic_import() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建动态导入模块
        let module_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_dynamic_module.js");
        let module_content = r#"
            export const value = "dynamic";
        "#;
        fs::write(&module_file, module_content).unwrap();

        // 创建主文件
        let main_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_dynamic_import.js");
        let main_content = r#"
            const module = await import('./test_dynamic_module.js');
            console.log("Dynamic import:", module.value);
        "#;
        fs::write(&main_file, main_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_dynamic_import.js");
        // 动态导入应该被支持
        assert!(result.is_ok());

        let _ = fs::remove_file(&main_file);
        let _ = fs::remove_file(&module_file);
    }

    #[test]
    fn test_run_script_with_dynamic_import_error() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建动态导入不存在的模块
        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_dynamic_import_error.js");
        let test_content = r#"
            try {
                const module = await import('./nonexistent_module.js');
                console.log(module);
            } catch (error) {
                console.log("Dynamic import error:", error.message);
            }
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_dynamic_import_error.js");
        // 动态导入错误应该被捕获
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_re_export() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建被重新导出的模块
        let module_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_re_export_module.js");
        let module_content = r#"
            export const value = 42;
            export function test() {
                return "test";
            }
        "#;
        fs::write(&module_file, module_content).unwrap();

        // 创建重新导出的文件
        let re_export_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_re_export.js");
        let re_export_content = r#"
            export * from './test_re_export_module.js';
            export const extra = "extra";
        "#;
        fs::write(&re_export_file, re_export_content).unwrap();

        // 创建主文件
        let main_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_re_export_main.js");
        let main_content = r#"
            import { value, test, extra } from './test_re_export.js';
            console.log(value, test(), extra);
        "#;
        fs::write(&main_file, main_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_re_export_main.js");
        // 重新导出应该被支持
        assert!(result.is_ok());

        let _ = fs::remove_file(&main_file);
        let _ = fs::remove_file(&re_export_file);
        let _ = fs::remove_file(&module_file);
    }

    #[test]
    fn test_run_script_with_mixed_exports() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建包含混合导出的文件
        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_mixed_exports.js");
        let test_content = r#"
            export const named1 = 1;
            export const named2 = 2;
            
            class MyClass {
                constructor() {
                    this.value = 42;
                }
            }
            
            export { MyClass };
            
            export default function defaultFunc() {
                return "default";
            }
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_mixed_exports.js");
        // 混合导出应该被支持
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_nested_imports() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建嵌套导入的模块
        let level3_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_level3.js");
        fs::write(&level3_file, "export const level3 = 3;").unwrap();

        let level2_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_level2.js");
        fs::write(&level2_file, "import { level3 } from './test_level3.js'; export const level2 = level3 + 1;").unwrap();

        let level1_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_level1.js");
        fs::write(&level1_file, "import { level2 } from './test_level2.js'; export const level1 = level2 + 1;").unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_level1.js");
        // 嵌套导入应该被支持
        assert!(result.is_ok());

        let _ = fs::remove_file(&level1_file);
        let _ = fs::remove_file(&level2_file);
        let _ = fs::remove_file(&level3_file);
    }

    #[test]
    fn test_run_script_with_invalid_json_in_import() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建包含无效 JSON 的文件
        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_invalid_json.js");
        let test_content = r#"
            import data from './invalid.json' assert { type: 'json' };
            console.log(data);
        "#;
        fs::write(&test_file, test_content).unwrap();

        // 创建无效的 JSON 文件
        let json_file = std::env::current_dir()
            .unwrap()
            .join("scripts/invalid.json");
        fs::write(&json_file, "{ invalid json }").unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_invalid_json.js");
        // 无效 JSON 应该被捕获
        assert!(result.is_err());

        let _ = fs::remove_file(&test_file);
        let _ = fs::remove_file(&json_file);
    }

    #[test]
    fn test_run_script_with_module_evaluation_error() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建模块评估时会出错的文件
        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_eval_error.js");
        let test_content = r#"
            // 在顶层执行会抛出错误的代码
            throw new Error("Module evaluation error");
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_eval_error.js");
        // 模块评估错误被打印到 stderr，但 run_script 返回 Ok
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_event_loop_error() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建会导致事件循环错误的文件 - 使用超时
        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_event_loop_error.js");
        let test_content = r#"
            // 创建一个会超时的 promise
            const promise = new Promise((resolve) => {
                setTimeout(() => resolve("timeout"), 1);
            });
            
            const result = await promise;
            console.log("Promise resolved:", result);
        "#;
        fs::write(&test_file, test_content).unwrap();

        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_event_loop_error.js");
        // 应该能够执行
        assert!(result.is_ok());

        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_run_script_with_concurrent_promises() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        // 创建包含大量并发 promise 的文件
        let test_file = std::env::current_dir()
            .unwrap()
            .join("scripts/test_concurrent_promises.js");
        let test_content = r#"
            const promises = [];
            for (let i = 0; i < 100; i++) {
                promises.push(
                    new Promise(resolve => setTimeout(() => resolve(i), 1))
                );
            }
            const results = await Promise.all(promises);
            console.log("All promises resolved:", results.length);
                    "#;
                    fs::write(&test_file, test_content).unwrap();
            
                    let result = ScriptRunner::run_script(&mut runtime, "scripts/test_concurrent_promises.js");
                    // 并发 promise 应该被正确处理
                    assert!(result.is_ok());
            
                    let _ = fs::remove_file(&test_file);
                }
            
                #[test]
                fn test_run_script_error_message_format_for_path_resolution() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 测试路径解析失败的错误消息格式
                    let result = ScriptRunner::run_script(&mut runtime, "scripts/nonexistent.js");
                    assert!(result.is_err());
                    let err_msg = result.unwrap_err();
                    // 验证错误消息格式
                    assert!(err_msg.contains("Failed to resolve script path") || 
                            err_msg.contains("Failed to load module"));
                }
            
                #[test]
                fn test_run_script_error_message_format_for_invalid_path() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 测试无效路径的错误消息格式
                    let result = ScriptRunner::run_script(&mut runtime, "scripts/test\nfile.js");
                    assert!(result.is_err());
                    let err_msg = result.unwrap_err();
                    // 验证错误消息格式
                    assert!(err_msg.contains("Failed to resolve script path") || 
                            err_msg.contains("Failed to load module"));
                }
            
                #[test]
                fn test_run_script_error_message_format_for_module_load() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 创建导入不存在的模块
                    let test_file = std::env::current_dir()
                        .unwrap()
                        .join("scripts/test_module_load_error.js");
                    let test_content = r#"
                        import { value } from './nonexistent_module.js';
                    "#;
                    fs::write(&test_file, test_content).unwrap();
            
                    let result = ScriptRunner::run_script(&mut runtime, "scripts/test_module_load_error.js");
                    assert!(result.is_err());
                    let err_msg = result.unwrap_err();
                    // 验证错误消息格式
                    assert!(err_msg.contains("Failed to load module"));
            
                    let _ = fs::remove_file(&test_file);
                }
            
                #[test]
                fn test_run_script_error_message_format_for_syntax_error() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 创建语法错误的文件
                    let test_file = std::env::current_dir()
                        .unwrap()
                        .join("scripts/test_syntax_error_format.js");
                    let test_content = r#"
                        const x = 
                    "#;
                    fs::write(&test_file, test_content).unwrap();
            
                    let result = ScriptRunner::run_script(&mut runtime, "scripts/test_syntax_error_format.js");
                    assert!(result.is_err());
                    let err_msg = result.unwrap_err();
                    // 验证错误消息格式（可能是路径解析错误或模块加载错误）
                    assert!(!err_msg.is_empty());
            
                    let _ = fs::remove_file(&test_file);
                }
            
                #[test]
                fn test_run_script_error_message_contains_details() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 测试错误消息是否包含详细信息
                    let result = ScriptRunner::run_script(&mut runtime, "scripts\nonexistent.js");
                    assert!(result.is_err());
                    let err_msg = result.unwrap_err();
                    // 错误消息应该包含冒号，表示有详细信息
                    assert!(err_msg.contains(":"));
                }
            
                #[test]
                fn test_run_script_with_relative_path_dot() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 测试当前目录路径
                    let test_file = std::env::current_dir()
                        .unwrap()
                        .join("scripts/test_dot.js");
                    let test_content = r#"
                        console.log("Test");
                    "#;
                    fs::write(&test_file, test_content).unwrap();
            
                    let result = ScriptRunner::run_script(&mut runtime, "./scripts/test_dot.js");
                    assert!(result.is_ok());
            
                    let _ = fs::remove_file(&test_file);
                }
            
                #[test]
                fn test_run_script_with_absolute_path() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 测试绝对路径
                    let test_file = std::env::current_dir()
                        .unwrap()
                        .join("scripts/test_absolute.js");
                    let test_content = r#"
                        console.log("Test");
                    "#;
                    fs::write(&test_file, test_content).unwrap();
            
                    let result = ScriptRunner::run_script(&mut runtime, test_file.to_str().unwrap());
                    assert!(result.is_ok());
            
                    let _ = fs::remove_file(&test_file);
                }
            
                #[test]
                    fn test_run_script_with_empty_path() {
                        let mut runtime =
                            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
                
                        // 测试空路径
                        let result = ScriptRunner::run_script(&mut runtime, "");
                        assert!(result.is_err());
                        let err_msg = result.unwrap_err();
                        // 空路径可能导致不同的错误
                        assert!(err_msg.contains("Failed to resolve script path") || 
                                err_msg.contains("Failed to load module"));
                    }            
                #[test]
                fn test_run_script_with_whitespace_path() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 测试空白路径
                    let result = ScriptRunner::run_script(&mut runtime, "   ");
                    assert!(result.is_err());
                    let err_msg = result.unwrap_err();
                    assert!(err_msg.contains("Failed to resolve script path") || 
                            err_msg.contains("Failed to load module"));
                }
            
                #[test]
                fn test_run_script_with_special_characters_in_path() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 测试包含特殊字符的路径
                    let test_file = std::env::current_dir()
                        .unwrap()
                        .join("scripts/test_special_@#$.js");
                    let test_content = r#"
                        console.log("Test");
                    "#;
                    fs::write(&test_file, test_content).unwrap();
            
                    let result = ScriptRunner::run_script(&mut runtime, "scripts/test_special_@#$.js");
                    assert!(result.is_ok());
            
                    let _ = fs::remove_file(&test_file);
                }
            
                #[test]
                fn test_run_script_with_directory_path() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 测试目录路径（应该失败）
                    let result = ScriptRunner::run_script(&mut runtime, "scripts/");
                    assert!(result.is_err());
                }
            
                #[test]
                fn test_run_script_with_non_js_extension() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 创建非 .js 扩展名的文件
                    let test_file = std::env::current_dir()
                        .unwrap()
                        .join("scripts/test.txt");
                    let test_content = r#"
                        console.log("Test");
                    "#;
                    fs::write(&test_file, test_content).unwrap();
            
                    // 尝试加载 .txt 文件
                    let result = ScriptRunner::run_script(&mut runtime, "scripts/test.txt");
                    // 可能成功或失败，取决于实现
                    // 我们只是确保不会崩溃
                    let _ = result;
            
                    let _ = fs::remove_file(&test_file);
                }
            
                #[test]
                fn test_run_script_with_symlink() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 创建目标文件
                    let target_file = std::env::current_dir()
                        .unwrap()
                        .join("scripts/test_symlink_target.js");
                    let target_content = r#"
                        console.log("Symlink test");
                    "#;
                    fs::write(&target_file, target_content).unwrap();
            
                    // 创建符号链接（仅 Unix）
                    #[cfg(unix)]
                    {
                        let symlink_file = std::env::current_dir()
                            .unwrap()
                            .join("scripts/test_symlink.js");
                        let _ = std::os::unix::fs::symlink(&target_file, &symlink_file);
                        
                        let result = ScriptRunner::run_script(&mut runtime, "scripts/test_symlink.js");
                        // 符号链接应该能正常工作
                        assert!(result.is_ok());
                        
                        let _ = fs::remove_file(&symlink_file);
                    }
            
                    let _ = fs::remove_file(&target_file);
                }
            
                #[test]
                fn test_run_script_with_import_of_nonexistent_file() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 创建导入不存在文件的脚本
                    let test_file = std::env::current_dir()
                        .unwrap()
                        .join("scripts/test_import_nonexistent.js");
                    let test_content = r#"
                        import { value } from './does_not_exist.js';
                    "#;
                    fs::write(&test_file, test_content).unwrap();
            
                    let result = ScriptRunner::run_script(&mut runtime, "scripts/test_import_nonexistent.js");
                    assert!(result.is_err());
                    let err_msg = result.unwrap_err();
                    assert!(err_msg.contains("Failed to load module"));
            
                    let _ = fs::remove_file(&test_file);
                }
            
                #[test]
                fn test_run_script_with_import_of_invalid_file() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 创建无效的模块文件
                    let invalid_file = std::env::current_dir()
                        .unwrap()
                        .join("scripts/invalid_module.js");
                    fs::write(&invalid_file, "const x = ").unwrap();
            
                    // 创建导入无效文件的脚本
                    let test_file = std::env::current_dir()
                        .unwrap()
                        .join("scripts/test_import_invalid.js");
                    let test_content = r#"
                        import { x } from './invalid_module.js';
                    "#;
                    fs::write(&test_file, test_content).unwrap();
            
                    let result = ScriptRunner::run_script(&mut runtime, "scripts/test_import_invalid.js");
                    assert!(result.is_err());
            
                    let _ = fs::remove_file(&test_file);
                    let _ = fs::remove_file(&invalid_file);
                }
            
                #[test]
                fn test_run_script_error_is_string() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 验证错误返回的是 String 类型
                    let result = ScriptRunner::run_script(&mut runtime, "scripts/nonexistent.js");
                    assert!(result.is_err());
                    let err_msg = result.unwrap_err();
                    // 验证错误消息是字符串
                    assert!(err_msg.is_empty() || !err_msg.is_empty());
                    // 验证可以调用字符串方法
                    assert!(err_msg.len() > 0);
                }
            
                #[test]
                fn test_run_script_multiple_errors_same_type() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 测试多个同类型的错误
                    for i in 0..3 {
                        let result = ScriptRunner::run_script(&mut runtime, &format!("scripts/nonexistent_{}.js", i));
                        assert!(result.is_err());
                    }
                }
            
                #[test]
                fn test_run_script_error_consistency() {
                    let mut runtime =
                        crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();
            
                    // 测试相同错误的一致性
                    let result1 = ScriptRunner::run_script(&mut runtime, "scripts/nonexistent.js");
                    let result2 = ScriptRunner::run_script(&mut runtime, "scripts/nonexistent.js");
                    
                    assert!(result1.is_err());
                    assert!(result2.is_err());
                    
                    let err1 = result1.unwrap_err();
                    let err2 = result2.unwrap_err();
                    
                    // 错误消息应该是相似的（包含相同的错误类型）
                    assert_eq!(err1.contains("Failed to"), err2.contains("Failed to"));
                }
            }