use axum::{
    Router,
    http::{header, HeaderValue},
};
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};
use std::path::PathBuf;

/// 静态资源目录配置
#[derive(Debug, Clone)]
pub struct StaticDirConfig {
    /// URL 路径前缀
    pub path_prefix: String,
    /// 文件系统目录路径
    pub dir_path: String,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 缓存控制头（秒）
    pub cache_max_age: u64,
}

impl StaticDirConfig {
    /// 创建新的静态目录配置
    pub fn new(path_prefix: &str, dir_path: &str) -> Self {
        Self {
            path_prefix: path_prefix.to_string(),
            dir_path: dir_path.to_string(),
            enable_compression: true,
            cache_max_age: 3600, // 默认 1 小时
        }
    }

    /// 设置是否启用压缩
    pub fn with_compression(mut self, enable: bool) -> Self {
        self.enable_compression = enable;
        self
    }

    /// 设置缓存最大时间（秒）
    pub fn with_cache_max_age(mut self, max_age: u64) -> Self {
        self.cache_max_age = max_age;
        self
    }
}

/// 静态服务器配置
#[derive(Debug, Clone)]
pub struct StaticServerConfig {
    /// 静态目录列表
    pub dirs: Vec<StaticDirConfig>,
    /// 默认首页文件
    pub index_file: String,
    /// 是否启用 CORS
    pub enable_cors: bool,
    /// 是否启用请求追踪
    pub enable_trace: bool,
}

impl Default for StaticServerConfig {
    fn default() -> Self {
        Self {
            dirs: vec![],
            index_file: "index.html".to_string(),
            enable_cors: true,
            enable_trace: false,
        }
    }
}

impl StaticServerConfig {
    /// 创建新的静态服务器配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加静态目录
    pub fn add_dir(mut self, config: StaticDirConfig) -> Self {
        self.dirs.push(config);
        self
    }

    /// 设置默认首页文件
    pub fn with_index_file(mut self, index_file: &str) -> Self {
        self.index_file = index_file.to_string();
        self
    }

    /// 设置是否启用 CORS
    pub fn with_cors(mut self, enable: bool) -> Self {
        self.enable_cors = enable;
        self
    }

    /// 设置是否启用请求追踪
    pub fn with_trace(mut self, enable: bool) -> Self {
        self.enable_trace = enable;
        self
    }

    /// 构建静态资源路由
    pub fn build_router(self) -> Router {
        let mut router = Router::new();

        // 添加每个静态目录
        for dir_config in self.dirs {
            let serve_dir = if dir_config.enable_compression {
                ServeDir::new(&dir_config.dir_path)
                    .precompressed_gzip()
                    .precompressed_br()
                    .precompressed_deflate()
            } else {
                ServeDir::new(&dir_config.dir_path)
            };

            // 添加缓存控制头
            let cache_header = format!("public, max-age={}", dir_config.cache_max_age);
            let cache_layer = SetResponseHeaderLayer::overriding(
                header::CACHE_CONTROL,
                HeaderValue::from_str(&cache_header).unwrap(),
            );

            // 构建带缓存和压缩的服务
            let static_service = tower::ServiceBuilder::new()
                .layer(cache_layer)
                .service(serve_dir);

            router = router.nest_service(&dir_config.path_prefix, static_service);
        }

        // 添加 fallback 服务（返回 index.html）
        let index_path = PathBuf::from("static").join(&self.index_file);
        let fallback_service = ServeFile::new(index_path);

        router = router.fallback_service(fallback_service);

        // 添加 CORS
        if self.enable_cors {
            let cors = CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
                .allow_headers([header::CONTENT_TYPE]);
            router = router.layer(cors);
        }

        // 添加请求追踪
        if self.enable_trace {
            router = router.layer(TraceLayer::new_for_http());
        }

        router
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_dir_config_creation() {
        let config = StaticDirConfig::new("/static", "static");
        assert_eq!(config.path_prefix, "/static");
        assert_eq!(config.dir_path, "static");
        assert!(config.enable_compression);
        assert_eq!(config.cache_max_age, 3600);
    }

    #[test]
    fn test_static_dir_config_builder() {
        let config = StaticDirConfig::new("/assets", "assets")
            .with_compression(false)
            .with_cache_max_age(7200);

        assert!(!config.enable_compression);
        assert_eq!(config.cache_max_age, 7200);
    }

    #[test]
    fn test_static_server_config_default() {
        let config = StaticServerConfig::new();
        assert_eq!(config.dirs.len(), 0);
        assert_eq!(config.index_file, "index.html");
        assert!(config.enable_cors);
        assert!(!config.enable_trace);
    }

    #[test]
    fn test_static_server_config_builder() {
        let config = StaticServerConfig::new()
            .add_dir(StaticDirConfig::new("/public", "public"))
            .with_index_file("home.html")
            .with_cors(false)
            .with_trace(true);

        assert_eq!(config.dirs.len(), 1);
        assert_eq!(config.index_file, "home.html");
        assert!(!config.enable_cors);
        assert!(config.enable_trace);
    }
}