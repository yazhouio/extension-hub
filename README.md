文件服务器和上传工具。

### v1: [grpc-server-and-client](./grpc-server-and-client/README.md)
    
    * 使用 axum 作为文件服务器，提升文件和上传功能。
    * 使用 tonic 提供 grpc，实现文件检测、文本替换等其他功能。
    * 使用 reqwest 实现 http 上传功能。

### v2: [sftp-upload-helper](./sftp-upload-helper/)
    
    1. 服务器（不用实现）
        * sftp 提供上传功能
        * nginx 提供文件服务器
    2. 客户端
        * opendal 实现文件上传
        * 自行实现文本替换功能