use deno_core::{ModuleLoader, ResolutionKind};
use deno_error::JsErrorBox;
use std::fs;
use std::path::PathBuf;
use super::super::loader::TsModuleLoader;

#[tokio::test]
async fn test_resolve_valid_module() {
    let loader = TsModuleLoader;
    let result = loader.resolve(
        "./test_module.js",
        "file:///scripts/test.js",
        ResolutionKind::Import,
    );
    assert!(result.is_ok());
    let specifier = result.unwrap();
    assert_eq!(specifier.scheme(), "file");
}

#[tokio::test]
async fn test_resolve_absolute_path() {
    let loader = TsModuleLoader;
    let result = loader.resolve(
        "file:///scripts/test.js",
        "file:///scripts/main.js",
        ResolutionKind::Import,
    );
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resolve_relative_path() {
    let loader = TsModuleLoader;
    let result = loader.resolve(
        "../parent.js",
        "file:///scripts/subdir/child.js",
        ResolutionKind::Import,
    );
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resolve_with_empty_referrer() {
    let loader = TsModuleLoader;
    let result = loader.resolve(
        "./test.js",
        "",
        ResolutionKind::Import,
    );
    assert!(result.is_err());
}

#[tokio::test]
async fn test_resolve_with_empty_specifier() {
    let loader = TsModuleLoader;
    let result = loader.resolve(
        "",
        "file:///scripts/test.js",
        ResolutionKind::Import,
    );
    assert!(result.is_err());
}

#[tokio::test]
async fn test_load_javascript_file() {
    let loader = TsModuleLoader;
    let test_file = std::env::current_dir().unwrap().join("scripts/test_loader.js");

    let test_content = "export const value = 42;";
    fs::write(&test_file, test_content).unwrap();

    let specifier = deno_core::ModuleSpecifier::from_file_path(&test_file).unwrap();
    let response = loader.load(&specifier, None, deno_core::ModuleLoadOptions {
        is_dynamic_import: false,
        is_synchronous: false,
        requested_module_type: deno_core::RequestedModuleType::None,
    });

    match response {
        deno_core::ModuleLoadResponse::Async(fut) => {
            let result = fut.await;
            assert!(result.is_ok());
            let module_source = result.unwrap();
            assert_eq!(module_source.module_type, deno_core::ModuleType::JavaScript);
        }
        _ => panic!("Expected async response"),
    }

    let _ = fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_load_typescript_file() {
    let loader = TsModuleLoader;
    let test_file = std::env::current_dir().unwrap().join("scripts/test_loader.ts");

    let test_content = "export const value: number = 42;";
    fs::write(&test_file, test_content).unwrap();

    let specifier = deno_core::ModuleSpecifier::from_file_path(&test_file).unwrap();
    let response = loader.load(&specifier, None, deno_core::ModuleLoadOptions {
        is_dynamic_import: false,
        is_synchronous: false,
        requested_module_type: deno_core::RequestedModuleType::None,
    });

    match response {
        deno_core::ModuleLoadResponse::Async(fut) => {
            let result = fut.await;
            assert!(result.is_ok());
            let module_source = result.unwrap();
            assert_eq!(module_source.module_type, deno_core::ModuleType::JavaScript);
        }
        _ => panic!("Expected async response"),
    }

    let _ = fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_load_tsx_file() {
    let loader = TsModuleLoader;
    let test_file = std::env::current_dir().unwrap().join("scripts/test_loader.tsx");

    let test_content = "export const element = <div>Hello</div>;";
    fs::write(&test_file, test_content).unwrap();

    let specifier = deno_core::ModuleSpecifier::from_file_path(&test_file).unwrap();
    let response = loader.load(&specifier, None, deno_core::ModuleLoadOptions {
        is_dynamic_import: false,
        is_synchronous: false,
        requested_module_type: deno_core::RequestedModuleType::None,
    });

    match response {
        deno_core::ModuleLoadResponse::Async(fut) => {
            let result = fut.await;
            assert!(result.is_ok());
            let module_source = result.unwrap();
            assert_eq!(module_source.module_type, deno_core::ModuleType::JavaScript);
        }
        _ => panic!("Expected async response"),
    }

    let _ = fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_load_nonexistent_file() {
    let loader = TsModuleLoader;
    let test_file = std::env::current_dir().unwrap().join("scripts/nonexistent_file.js");

    let specifier = deno_core::ModuleSpecifier::from_file_path(&test_file).unwrap();
    let response = loader.load(&specifier, None, deno_core::ModuleLoadOptions {
        is_dynamic_import: false,
        is_synchronous: false,
        requested_module_type: deno_core::RequestedModuleType::None,
    });

    match response {
        deno_core::ModuleLoadResponse::Async(fut) => {
            let result = fut.await;
            assert!(result.is_err());
        }
        _ => panic!("Expected async response"),
    }
}

#[tokio::test]
async fn test_load_invalid_file_url() {
    let loader = TsModuleLoader;
    let specifier = deno_core::ModuleSpecifier::parse("http://example.com/module.js").unwrap();
    let response = loader.load(&specifier, None, deno_core::ModuleLoadOptions {
        is_dynamic_import: false,
        is_synchronous: false,
        requested_module_type: deno_core::RequestedModuleType::None,
    });

    match response {
        deno_core::ModuleLoadResponse::Async(fut) => {
            let result = fut.await;
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("Only file:// URLs are supported"));
        }
        _ => panic!("Expected async response"),
    }
}

#[tokio::test]
async fn test_load_syntax_error_typescript() {
    let loader = TsModuleLoader;
    let test_file = std::env::current_dir().unwrap().join("scripts/test_syntax_error.ts");

    let test_content = "export const value: number = ;";
    fs::write(&test_file, test_content).unwrap();

    let specifier = deno_core::ModuleSpecifier::from_file_path(&test_file).unwrap();
    let response = loader.load(&specifier, None, deno_core::ModuleLoadOptions {
        is_dynamic_import: false,
        is_synchronous: false,
        requested_module_type: deno_core::RequestedModuleType::None,
    });

    match response {
        deno_core::ModuleLoadResponse::Async(fut) => {
            let result = fut.await;
            assert!(result.is_err());
        }
        _ => panic!("Expected async response"),
    }

    let _ = fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_load_empty_javascript_file() {
    let loader = TsModuleLoader;
    let test_file = std::env::current_dir().unwrap().join("scripts/test_empty.js");

    fs::write(&test_file, "").unwrap();

    let specifier = deno_core::ModuleSpecifier::from_file_path(&test_file).unwrap();
    let response = loader.load(&specifier, None, deno_core::ModuleLoadOptions {
        is_dynamic_import: false,
        is_synchronous: false,
        requested_module_type: deno_core::RequestedModuleType::None,
    });

    match response {
        deno_core::ModuleLoadResponse::Async(fut) => {
            let result = fut.await;
            assert!(result.is_ok());
            let module_source = result.unwrap();
            assert_eq!(module_source.module_type, deno_core::ModuleType::JavaScript);
        }
        _ => panic!("Expected async response"),
    }

    let _ = fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_load_javascript_with_special_characters() {
    let loader = TsModuleLoader;
    let test_file = std::env::current_dir().unwrap().join("scripts/test_special.js");

    let test_content = "export const str = 'Hello\nWorld\t!';";
    fs::write(&test_file, test_content).unwrap();

    let specifier = deno_core::ModuleSpecifier::from_file_path(&test_file).unwrap();
    let response = loader.load(&specifier, None, deno_core::ModuleLoadOptions {
        is_dynamic_import: false,
        is_synchronous: false,
        requested_module_type: deno_core::RequestedModuleType::None,
    });

    match response {
        deno_core::ModuleLoadResponse::Async(fut) => {
            let result = fut.await;
            assert!(result.is_ok());
            let module_source = result.unwrap();
            assert_eq!(module_source.module_type, deno_core::ModuleType::JavaScript);
        }
        _ => panic!("Expected async response"),
    }

    let _ = fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_load_typescript_with_types() {
    let loader = TsModuleLoader;
    let test_file = std::env::current_dir().unwrap().join("scripts/test_types.ts");

    let test_content = r#"
        interface User {
            name: string;
            age: number;
        }

        export const user: User = { name: "Alice", age: 30 };
    "#;
    fs::write(&test_file, test_content).unwrap();

    let specifier = deno_core::ModuleSpecifier::from_file_path(&test_file).unwrap();
    let response = loader.load(&specifier, None, deno_core::ModuleLoadOptions {
        is_dynamic_import: false,
        is_synchronous: false,
        requested_module_type: deno_core::RequestedModuleType::None,
    });

    match response {
        deno_core::ModuleLoadResponse::Async(fut) => {
            let result = fut.await;
            assert!(result.is_ok());
            let module_source = result.unwrap();
            assert_eq!(module_source.module_type, deno_core::ModuleType::JavaScript);
        }
        _ => panic!("Expected async response"),
    }

    let _ = fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_load_mts_file() {
    let loader = TsModuleLoader;
    let test_file = std::env::current_dir().unwrap().join("scripts/test_loader.mts");

    let test_content = "export const value = 42;";
    fs::write(&test_file, test_content).unwrap();

    let specifier = deno_core::ModuleSpecifier::from_file_path(&test_file).unwrap();
    let response = loader.load(&specifier, None, deno_core::ModuleLoadOptions {
        is_dynamic_import: false,
        is_synchronous: false,
        requested_module_type: deno_core::RequestedModuleType::None,
    });

    match response {
        deno_core::ModuleLoadResponse::Async(fut) => {
            let result = fut.await;
            assert!(result.is_ok());
            let module_source = result.unwrap();
            assert_eq!(module_source.module_type, deno_core::ModuleType::JavaScript);
        }
        _ => panic!("Expected async response"),
    }

    let _ = fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_load_cts_file() {
    let loader = TsModuleLoader;
    let test_file = std::env::current_dir().unwrap().join("scripts/test_loader.cts");

    let test_content = "export const value = 42;";
    fs::write(&test_file, test_content).unwrap();

    let specifier = deno_core::ModuleSpecifier::from_file_path(&test_file).unwrap();
    let response = loader.load(&specifier, None, deno_core::ModuleLoadOptions {
        is_dynamic_import: false,
        is_synchronous: false,
        requested_module_type: deno_core::RequestedModuleType::None,
    });

    match response {
        deno_core::ModuleLoadResponse::Async(fut) => {
            let result = fut.await;
            assert!(result.is_ok());
            let module_source = result.unwrap();
            assert_eq!(module_source.module_type, deno_core::ModuleType::JavaScript);
        }
        _ => panic!("Expected async response"),
    }

    let _ = fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_load_jsx_file() {
    let loader = TsModuleLoader;
    let test_file = std::env::current_dir().unwrap().join("scripts/test_loader.jsx");

    let test_content = "export const element = <div>Hello</div>;";
    fs::write(&test_file, test_content).unwrap();

    let specifier = deno_core::ModuleSpecifier::from_file_path(&test_file).unwrap();
    let response = loader.load(&specifier, None, deno_core::ModuleLoadOptions {
        is_dynamic_import: false,
        is_synchronous: false,
        requested_module_type: deno_core::RequestedModuleType::None,
    });

    match response {
        deno_core::ModuleLoadResponse::Async(fut) => {
            let result = fut.await;
            assert!(result.is_ok());
            let module_source = result.unwrap();
            assert_eq!(module_source.module_type, deno_core::ModuleType::JavaScript);
        }
        _ => panic!("Expected async response"),
    }

    let _ = fs::remove_file(&test_file);
}