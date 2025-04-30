use anyhow::Result;
use common::config::AppConfig;
use clap::Parser;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

mod model;
mod repository;
mod service;

use service::group_service::GroupServiceImpl;
use common::proto::group::group_service_server::GroupServiceServer;

#[derive(Parser, Debug)]
#[clap(name = "group-service", about = "群组服务")]
struct Args {
    /// 配置文件路径
    #[clap(short, long, default_value = ".env")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化命令行参数
    let args = Args::parse();
    
    // 加载.env文件
    dotenv::from_path(&args.config).ok();
    
    // 初始化日志
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    // 加载配置
    let config = AppConfig::new("group-service")?;
    let addr = format!("{}:{}", config.service.host, config.service.port).parse::<SocketAddr>()?;
    
    // 初始化数据库连接池
    let db_pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database.url)
        .await 
    {
        Ok(pool) => {
            info!("数据库连接成功");
            pool
        }
        Err(err) => {
            error!("数据库连接失败: {}", err);
            return Err(err.into());
        }
    };
    
    // 运行数据库迁移
    if let Err(err) = run_migrations(&db_pool).await {
        error!("数据库迁移失败: {}", err);
        return Err(err);
    }
    
    // 初始化群组服务
    let group_service = GroupServiceImpl::new(db_pool);
    
    // 启动gRPC服务
    info!("群组服务启动，监听地址: {}", addr);
    Server::builder()
        .add_service(GroupServiceServer::new(group_service))
        .serve(addr)
        .await?;
    
    Ok(())
}

async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("运行数据库迁移...");
    
    // 创建groups表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS groups (
            id UUID PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            description VARCHAR(500),
            avatar_url VARCHAR(255),
            owner_id UUID NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;
    
    // 创建group_members表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS group_members (
            id UUID PRIMARY KEY,
            group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
            user_id UUID NOT NULL,
            role INTEGER NOT NULL,
            joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(group_id, user_id)
        );
        "#,
    )
    .execute(pool)
    .await?;
    
    info!("数据库迁移完成");
    Ok(())
}