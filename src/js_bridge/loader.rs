use deno_ast::{MediaType, ParseParams};
use deno_core::{
    ModuleLoadOptions, ModuleLoadResponse, ModuleLoader, ModuleSource, ModuleSourceCode,
    ModuleSpecifier, ModuleType, ResolutionKind,
};
use std::sync::Arc;

pub struct TsModuleLoader;

impl ModuleLoader for TsModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, deno_error::JsErrorBox> {
        deno_core::resolve_import(specifier, referrer).map_err(deno_error::JsErrorBox::from_err)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&deno_core::ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        let module_specifier = module_specifier.clone();
        let fut = async move {
            let path = module_specifier
                .to_file_path()
                .map_err(|_| deno_error::JsErrorBox::generic("Only file:// URLs are supported"))?;

            let media_type = MediaType::from_path(&path);
            let code = std::fs::read_to_string(&path).map_err(deno_error::JsErrorBox::from_err)?;

            let (transpiled_code, module_type) = match media_type {
                MediaType::TypeScript
                | MediaType::Mts
                | MediaType::Cts
                | MediaType::Jsx
                | MediaType::Tsx => {
                    let parsed = deno_ast::parse_module(ParseParams {
                        specifier: module_specifier.clone(),
                        text: Arc::from(code),
                        media_type,
                        capture_tokens: false,
                        scope_analysis: false,
                        maybe_syntax: None,
                    })
                    .map_err(deno_error::JsErrorBox::from_err)?;
                    let transpiled = parsed
                        .transpile(
                            &deno_ast::TranspileOptions {
                                ..Default::default()
                            },
                            &deno_ast::TranspileModuleOptions::default(),
                            &deno_ast::EmitOptions::default(),
                        )
                        .map_err(deno_error::JsErrorBox::from_err)?
                        .into_source();
                    (transpiled.text, ModuleType::JavaScript)
                }
                _ => (code, ModuleType::JavaScript),
            };

            Ok(ModuleSource::new(
                module_type,
                ModuleSourceCode::String(transpiled_code.into()),
                &module_specifier,
                None,
            ))
        };
        ModuleLoadResponse::Async(Box::pin(fut))
    }
}
