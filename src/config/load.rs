use config::{ConfigError, File};
use crate::config::Config;

pub fn load_config(config_path: &str) -> Result<Config, ConfigError> {
    let settings = match config::Config::builder()
        // 从传入的路径加载配置文件，`required(true)` 表示文件必须存在
        .add_source(File::with_name(config_path).required(true))
        // 构建配置实例
        .build() {
        Ok(settings) => {
            settings
        },
        Err(e) => {
            return Err(e);
        }
    };

    // 尝试将加载的配置反序列化为 `Config` 结构体
    match settings.try_deserialize::<Config>() {
        Ok(config) => {
            Ok(config)
        },
        Err(e) => {
            Err(e)
        }
    }
}