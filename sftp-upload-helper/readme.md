### sftp 上传助手

#### 1. 配置
 - 1.1 配置文件
    - 1.1.1 配置文件路径
        * ssh key 文件路径:
         `~/.ssh/sftp.id_rsa`
         `./sftp.id_rsa`
         `config.toml` 中 `key_file` 字段
        > 优先级：`config.toml/key_file` > `./sftp.id_rsa` > `~/.ssh/sftp.id_rsa`
        * 配置文件路径：
            * config 默认值
            * `~/.config/sftp-upload-helper/config.toml`
            * `./config.toml`
        > 优先级：三处文件同时生效。`./config.toml` > `~/.config/sftp-upload-helper/config.toml` > `config 默认值`，存在则覆盖

    - 1.1.2 配置文件格式
```toml
# Config
origin_dir = "your_origin_dir"

# UploadConfig
[upload_config]
path_ignore_globs = ["*/target/**"]
path_globs = ["*"]
target_dir = "your_target_dir"

# Server as Ssh
[upload_config.server.ssh]
endpoint = "your_endpoint"
user = "your_user"
key_file = "your_key_file"

# Server as LocalFs (uncomment if needed)
[upload_config.server.local_fs]
path = "/tmp"

# TextReplaceConfig
[[text_replace_config]]
items = [
    { target = "target_string", origin = "origin_string" },
]
path_ignore_globs = ["ignore_pattern", "ignore_pattern"]
path_globs = ["pattern4", "pattern4"]
```

### 1.2 配置项说明

#### 1.2.1 upload_config
- `path_ignore_globs`: 要忽略的路径匹配模式
- `path_globs`: 要匹配的路径模式
- `target_dir`: 上传目标目录

#### 1.2.2 server
- **ssh**
  - `endpoint`: sftp 服务器地址
  - `user`: sftp 用户名
  - `key_file`: ssh key 文件路径
- **local_fs**
  - `path`: 本地文件系统路径

#### 1.2.3 text_replace_config
- `items`: 文本替换项
  - `target`: 替换目标
  - `origin`: 替换源
- `path_ignore_globs`: 要忽略的路径匹配模式
- `path_globs`: 要匹配的路径模式