use config::{Config, ConfigError, File, FileFormat};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::path::Path;
use std::thread;
use tracing::{info, warn, error};

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub topic_private_messages: String,
    pub topic_group_messages: String,
    pub consumer_group_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServiceConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OssConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub avatar_bucket: String,
    pub region: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub kafka: KafkaConfig,
    pub jwt: JwtConfig,
    pub service: ServiceConfig,
    pub oss: OssConfig,
}

// 封装配置以支持动态更新
pub struct DynamicConfig {
    current: RwLock<Arc<AppConfig>>,
    config_paths: Vec<String>,
    service_name: String,
    refresh_interval: Duration,
}

impl AppConfig {
    // 创建一个新的AppConfig实例
    pub fn new(service_name: &str) -> Result<Self, ConfigError> {
        Self::load_from_sources(service_name, None)
    }

    // 从文件加载配置
    pub fn from_file(service_name: &str, file_path: &str) -> Result<Self, ConfigError> {
        Self::load_from_sources(service_name, Some(file_path))
    }

    // 从多个来源加载配置
    fn load_from_sources(service_name: &str, file_path: Option<&str>) -> Result<Self, ConfigError> {
        // 尝试加载.env文件，但不要求它必须存在
        dotenv().ok();
        
        // 开始构建配置
        let mut builder = Config::builder();
        
        // 1. 默认配置
        builder = builder.set_default("service.host", "127.0.0.1")?
            .set_default("service.port", 8000)?
            .set_default("database.url", "postgres://kelisi:123456@localhost:5432/rustim")?
            .set_default("redis.url", "redis://localhost:6379")?
            .set_default("kafka.bootstrap_servers", "localhost:29092")?
            .set_default("kafka.topic_private_messages", "private_messages")?
            .set_default("kafka.topic_group_messages", "group_messages")?
            .set_default("kafka.consumer_group_id", "rustim_consumer_group")?
            .set_default("jwt.secret", "default_jwt_secret")?
            .set_default("jwt.expiration", 86400)?
            .set_default("oss.endpoint", "")?
            .set_default("oss.access_key", "")?
            .set_default("oss.secret_key", "")?
            .set_default("oss.bucket", "")?
            .set_default("oss.avatar_bucket", "")?
            .set_default("oss.region", "")?;
        
        // 2. 配置文件 (如果指定)
        if let Some(path) = file_path {
            if Path::new(path).exists() {
                let format = if path.ends_with(".json") {
                    FileFormat::Json
                } else if path.ends_with(".yaml") || path.ends_with(".yml") {
                    FileFormat::Yaml
                } else {
                    FileFormat::Toml
                };
                
                builder = builder.add_source(File::with_name(path).format(format));
            }
        }
        
        // 3. 检查默认的配置文件路径
        for path in ["config.toml", "config.yaml", "config.yml", "config.json", ".env"] {
            if Path::new(path).exists() {
                let format = if path.ends_with(".json") {
                    FileFormat::Json
                } else if path.ends_with(".yaml") || path.ends_with(".yml") {
                    FileFormat::Yaml
                } else if path.ends_with(".toml") {
                    FileFormat::Toml
                } else {
                    // .env 文件默认使用环境变量格式
                    continue;
                };
                
                builder = builder.add_source(File::with_name(path).format(format));
            }
        }
        
        // 4. 读取环境变量 (最高优先级)
        builder = builder.add_source(config::Environment::default().separator("_"));
        
        // 构建配置
        let config = builder.build()?;
        
        // 提取服务特定的配置
        let host_env = format!("{}_HOST", service_name.to_uppercase().replace("-", "_"));
        let port_env = format!("{}_PORT", service_name.to_uppercase().replace("-", "_"));
        
        let host = config.get_string(&host_env).unwrap_or_else(|_| {
            config.get_string("service.host").unwrap_or_else(|_| "127.0.0.1".to_string())
        });
        
        let port = config.get_int(&port_env).unwrap_or_else(|_| {
            config.get_int("service.port").unwrap_or(8000)
        }) as u16;
        
        // 构建AppConfig实例
        let database_url = config.get_string("database.url").unwrap_or_else(|_| {
            "postgres://kelisi:123456@localhost:5432/rustim".to_string()
        });
        
        let redis_url = config.get_string("redis.url").unwrap_or_else(|_| {
            "redis://localhost:6379".to_string()
        });
        
        let kafka_bootstrap_servers = config.get_string("kafka.bootstrap_servers").unwrap_or_else(|_| {
            "localhost:29092".to_string()
        });
        
        let kafka_topic_private_messages = config.get_string("kafka.topic_private_messages").unwrap_or_else(|_| {
            "private_messages".to_string()
        });
        
        let kafka_topic_group_messages = config.get_string("kafka.topic_group_messages").unwrap_or_else(|_| {
            "group_messages".to_string()
        });
        
        let kafka_consumer_group_id = config.get_string("kafka.consumer_group_id").unwrap_or_else(|_| {
            "rustim_consumer_group".to_string()
        });
        
        let jwt_secret = config.get_string("jwt.secret").unwrap_or_else(|_| {
            "default_jwt_secret".to_string()
        });
        
        let jwt_expiration = config.get_int("jwt.expiration").unwrap_or(86400) as u64;
        
        let oss_endpoint = config.get_string("oss.endpoint").unwrap_or_default();
        let oss_access_key = config.get_string("oss.access_key").unwrap_or_default();
        let oss_secret_key = config.get_string("oss.secret_key").unwrap_or_default();
        let oss_bucket = config.get_string("oss.bucket").unwrap_or_default();
        let oss_avatar_bucket = config.get_string("oss.avatar_bucket").unwrap_or_default();
        let oss_region = config.get_string("oss.region").unwrap_or_default();
        
        Ok(AppConfig {
            database: DatabaseConfig { url: database_url },
            redis: RedisConfig { url: redis_url },
            kafka: KafkaConfig { 
                bootstrap_servers: kafka_bootstrap_servers,
                topic_private_messages: kafka_topic_private_messages,
                topic_group_messages: kafka_topic_group_messages,
                consumer_group_id: kafka_consumer_group_id,
            },
            jwt: JwtConfig {
                secret: jwt_secret,
                expiration: jwt_expiration,
            },
            service: ServiceConfig { host, port },
            oss: OssConfig {
                endpoint: oss_endpoint,
                access_key: oss_access_key,
                secret_key: oss_secret_key,
                bucket: oss_bucket,
                avatar_bucket: oss_avatar_bucket,
                region: oss_region,
            },
        })
    }
}

impl DynamicConfig {
    // 创建一个新的动态配置实例
    pub fn new(service_name: &str, config_paths: Vec<String>, refresh_interval_secs: u64) -> Result<Self, ConfigError> {
        let interval = Duration::from_secs(refresh_interval_secs);
        let config = AppConfig::new(service_name)?;
        
        Ok(DynamicConfig {
            current: RwLock::new(Arc::new(config)),
            config_paths,
            service_name: service_name.to_string(),
            refresh_interval: interval,
        })
    }
    
    // 获取当前配置
    pub fn get_config(&self) -> Arc<AppConfig> {
        self.current.read().unwrap().clone()
    }
    
    // 启动配置监控线程
    pub fn start_refresh_task(self: Arc<Self>) {
        let dynamic_config = self.clone();
        
        thread::spawn(move || {
            info!("配置监控线程启动，刷新间隔: {:?}", dynamic_config.refresh_interval);
            
            loop {
                thread::sleep(dynamic_config.refresh_interval);
                match dynamic_config.refresh_config() {
                    Ok(_) => info!("配置已更新"),
                    Err(e) => error!("刷新配置失败: {}", e),
                }
            }
        });
    }
    
    // 刷新配置
    fn refresh_config(&self) -> Result<(), ConfigError> {
        for path in &self.config_paths {
            if !Path::new(path).exists() {
                continue;
            }
            
            // 尝试从配置文件加载新配置
            match AppConfig::from_file(&self.service_name, path) {
                Ok(new_config) => {
                    // 更新当前配置
                    let mut current = self.current.write().unwrap();
                    *current = Arc::new(new_config);
                    info!("已从文件 {} 加载新配置", path);
                    return Ok(());
                }
                Err(e) => {
                    warn!("从 {} 加载配置失败: {}", path, e);
                }
            }
        }
        
        // 如果所有路径都失败，尝试从环境变量加载
        match AppConfig::new(&self.service_name) {
            Ok(new_config) => {
                let mut current = self.current.write().unwrap();
                *current = Arc::new(new_config);
                info!("已从环境变量加载新配置");
                Ok(())
            }
            Err(e) => {
                error!("刷新配置失败: {}", e);
                Err(e)
            }
        }
    }
} 