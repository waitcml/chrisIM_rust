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

use service::friend_service::FriendServiceImpl;
use common::proto::friend::friend_service_server::FriendServiceServer;

#[derive(Parser, Debug)]
#[clap(name = "friend-service", about = "好友关系服务")]
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
    let config = AppConfig::new("friend-service")?;
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
    
    // 初始化好友服务
    let friend_service = FriendServiceImpl::new(db_pool);
    
    // 启动gRPC服务
    info!("好友服务启动，监听地址: {}", addr);
    Server::builder()
        .add_service(FriendServiceServer::new(friend_service))
        .serve(addr)
        .await?;
    
    Ok(())
}

async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("运行数据库迁移...");
    
    // 创建friendships表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS friendships (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL,
            friend_id UUID NOT NULL,
            status INTEGER NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(user_id, friend_id)
        );
        "#,
    )
    .execute(pool)
    .await?;
    
    info!("数据库迁移完成");
    Ok(())
}