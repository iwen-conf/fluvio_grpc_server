mod config;
mod services;
mod proto;
mod handlers;

use std::sync::Arc;
use fluvio::Fluvio;
use tonic::transport::Server;
use console::{style, Emoji};
use figlet_rs::FIGfont;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 彩色 ASCII 艺术字 banner
    let standard_font = FIGfont::standard().unwrap();
    let figure = standard_font.convert("Fluvio gRPC").unwrap();
    println!("{}", style(figure).cyan().bold());

    // 1. 加载配置文件
    let config = config::load::load_config("src/config/config.toml")?;
    let host = &config.server.host;
    let port = config.server.port;
    let addr = format!("{}:{}", host, port).parse()?;

    // 2. 初始化 Fluvio 客户端
    println!(
        "{} {}",
        style(Emoji("🚀", "")).yellow(),
        style("正在连接 Fluvio 集群...").yellow()
    );
    let fluvio = match Fluvio::connect().await {
        Ok(f) => {
            println!(
                "{} {}",
                style(Emoji("✅", "")).green(),
                style("Fluvio 连接成功！").green().bold()
            );
            Arc::new(f)
        }
        Err(e) => {
            println!(
                "{} {}: {}",
                style(Emoji("❌", "")).red(),
                style("Fluvio 连接失败").red().bold(),
                style(&e).red()
            );
            return Err(anyhow::Error::from(e).context("Failed to connect to Fluvio"));
        }
    };

    // 3. 构建 gRPC 服务实例
    let fluvio_service = services::fluvio_server::FluvioServerService::new(fluvio.clone());

    println!(
        "{} {} {}",
        style(Emoji("🔊", "")).blue(),
        style("gRPC 服务已启动，监听地址:").blue(),
        style(addr).cyan().bold()
    );

    // 4. 启动 tonic gRPC 服务器
    Server::builder()
        .add_service(proto::fluvio_server::fluvio_service_server::FluvioServiceServer::new(fluvio_service))
        .serve(addr)
        .await
        .map_err(anyhow::Error::from)?;

    Ok(())
}
