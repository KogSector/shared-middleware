//! Logging configuration and initialization

use tracing::{info, Level};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};
use tracing_appender::{non_blocking, rolling};
use std::env;
use std::path::PathBuf;

/// Configuration for logging system
pub struct LoggingConfig {
    pub level: Level,
    pub json_format: bool,
    pub file_logging: bool,
    pub log_directory: PathBuf,
    pub service_name: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            json_format: false,
            file_logging: true,
            log_directory: PathBuf::from("logs"),
            service_name: "confuse-service".to_string(),
        }
    }
}

impl LoggingConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(level_str) = env::var("RUST_LOG_LEVEL") {
            config.level = match level_str.to_lowercase().as_str() {
                "trace" => Level::TRACE,
                "debug" => Level::DEBUG,
                "info" => Level::INFO,
                "warn" => Level::WARN,
                "error" => Level::ERROR,
                _ => Level::INFO,
            };
        }
        
        if env::var("ENVIRONMENT").unwrap_or_default() == "production" {
            config.json_format = true;
            config.level = Level::INFO; 
        }
        
        if let Ok(service) = env::var("SERVICE_NAME") {
            config.service_name = service;
        }
        
        config
    }
}

/// Initialize logging system
pub fn init_logging(config: LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    if config.file_logging {
        std::fs::create_dir_all(&config.log_directory)?;
    }
    
    let env_filter = EnvFilter::new(format!("{}={},confuse_common={}", 
        config.service_name.replace("-", "_"), 
        config.level,
        config.level
    ));
    
    let registry = Registry::default().with(env_filter);
    
    // Console layer
    let console_layer = fmt::layer()
        .pretty()
        .with_target(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true);
        
    if config.file_logging {
        let file_appender = rolling::daily(&config.log_directory, format!("{}.log", config.service_name));
        let (non_blocking_file, _guard) = non_blocking(file_appender);
        
        // We need to keep _guard alive for logs to be written, but we can't return it easily here
        // For now, allow it to be dropped means logs might not flush on panic, but OK for now.
        // Ideally, we should return a guard or init in main.
        // Correction: non_blocking returns a WorkerGuard that MUST be held.
        // Since we can't return it easily without changing signature, we might need a different approach.
        // Or we just accept console logging for now and minimal file logging.
        
        let file_layer = fmt::layer()
            .with_ansi(false)
            .with_writer(non_blocking_file);
            
        if config.json_format {
             registry.with(console_layer).with(file_layer.json()).init();
        } else {
             registry.with(console_layer).with(file_layer).init();
        }
        
        // Leak guard to keep file logging alive? Not ideal but common in init functions.
        Box::leak(Box::new(_guard)); 
    } else {
        registry.with(console_layer).init();
    }
    
    info!(
        service = %config.service_name,
        log_level = %config.level,
        "Logging initialized"
    );
    
    Ok(())
}
