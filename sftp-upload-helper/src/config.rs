use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 主配置结构体，支持命令行解析和序列化
#[derive(Parser, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// 目标目录
    #[arg(long)]
    #[serde(rename = "origin_dir")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub origin_dir: Option<String>,

    /// 上传配置
    #[clap(flatten)]
    #[serde(rename = "upload_config")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub upload_config: Option<UploadConfig>,

    /// 文本替换配置
    #[serde(rename = "text_replace_config")]
    #[arg(skip)] // clap 不支持直接解析 Vec，需要手动处理或通过文件解析
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    // 避免序列化空值, 避免 clap None 覆盖已有值
    pub text_replace_config: Option<Vec<TextReplaceConfig>>,
}

/// 上传配置结构体
#[derive(Parser, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UploadConfig {
    /// 服务器配置
    #[clap(subcommand)]
    pub server: Option<Server>,

    /// 上传目录
    #[arg(long)]
    #[serde(
        rename = "target_dir",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    pub target_dir: Option<String>,

    /// 要忽略的路径匹配模式
    #[arg(long, num_args(1..), use_value_delimiter = true)]
    pub path_ignore_globs: Option<Vec<String>>,

    /// 要匹配的路径模式
    #[arg(long, num_args(1..), use_value_delimiter = true)]
    pub path_globs: Option<Vec<String>>,
}

/// 服务器配置枚举
#[derive(Subcommand, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Server {
    /// SSH 配置
    Ssh(Ssh),

    /// 本地文件系统配置
    LocalFs(LocalFs),
}

/// SSH 服务器配置
#[derive(Parser, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Ssh {
    /// 服务器地址
    #[arg(long)]
    pub endpoint: String,

    /// 用户名
    #[arg(long)]
    pub user: String,

    /// 私钥文件路径
    #[arg(long)]
    pub key_file: String,
}

/// 本地文件系统配置
#[derive(Parser, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalFs {
    /// 本地文件路径
    #[arg(long, default_value = "/tmp")]
    pub path: String,
}

/// 文本替换
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TextReplaceConfigItem {
    /// 替换目标
    pub target: String,

    /// 替换来源
    pub origin: String,
}

/// 文本替换配置
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TextReplaceConfig {
    #[serde(default)]
    /// 文本替换配置项
    pub items: Vec<TextReplaceConfigItem>,

    /// 要忽略的路径匹配模式
    #[serde(default)]
    pub path_ignore_globs: Vec<String>,

    /// 要匹配的路径模式
    #[serde(default)]
    pub path_globs: Vec<String>,
}

// impl Default for LocalFs {
//     fn default() -> Self {
//         LocalFs {
//             path: "/tmp".to_string(),
//         }
//     }
// }
// impl Default for Server {
//     fn default() -> Self {
//         Server::LocalFs(LocalFs::default())
//     }
// }

impl Config {
    pub fn load() -> Result<Self> {
        let mut config = Figment::new();
        config = config.merge(Serialized::defaults(Config::default()));
        let home_config =
            shellexpand::tilde("~/.config/sftp-upload-helper/config.toml").into_owned();
        if Path::new(&home_config).exists() {
            config = config.merge(Toml::file(home_config));
        }
        // 获取运行目录的 config.yaml
        let current_dir = std::env::current_dir()?;
        let current_dir_config = current_dir.join("config.toml");

        if current_dir_config.exists() {
            config = config.merge(Toml::file(current_dir_config));
        }
        config = config.merge(Serialized::defaults(Config::parse()));
        config.extract().context("Failed to extract configuration")
    }

    pub fn validate(&self) -> Result<()> {
        match self.origin_dir {
            Some(ref origin_dir) => {
                if !Path::new(origin_dir).exists() {
                    anyhow::bail!("Origin directory not exists: {}", origin_dir);
                }
            }
            None => {
                anyhow::bail!("Origin directory not set.");
            }
        }
        match self.upload_config {
            Some(ref upload_config) => match upload_config.server {
                Some(_) => {}
                None => {
                    anyhow::bail!("Server configuration not set.");
                }
            },
            None => {
                anyhow::bail!("Upload configuration not set.");
            }
        }
        Ok(())
    }
}
