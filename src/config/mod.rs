use serde::Deserialize;

pub mod load;

// 服务配置结构体
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// 服务器绑定的主机地址。
    pub host: String,
    /// 服务器监听的端口号。
    pub port: u16,
}