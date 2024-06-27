# plugin-hub

分为服务端和客户端，分别负责文件的发送和接收。
![image](https://github.com/yazhouio/plugin-hub/assets/17949154/72e2d576-1921-4305-87eb-56dc2e596227)

大体流程如图。
![image](https://github.com/yazhouio/plugin-hub/assets/17949154/e1b30011-6bcb-430f-8d08-3c365bb22ec5)


### server 提供功能
| TODO | 功能 | |
| --- | --- | --- |
| <ul><li>- [x] </li></ul> | 根据 tar hash 判断是否存在 | grpc |
| <ul><li>- [x] </li></ul> | 根据配置，获取上传地址 | grpc |
| <ul><li>- [x] </li></ul> | http 上传 tar 包 | http |
| <ul><li>- [x] </li></ul> | http 下载 tar 包 | http |
| <ul><li>- [x] </li></ul> | 请求解压 tar 包到指定目录 | grpc |
| <ul><li>- [x] </li></ul> | 指定文件夹文本替换 | grpc |
| <ul><li>- [ ] </li></ul> | 未使用文件清理｜ grpc ｜

## TODO: server 额外功能（待定）
在 server 启动时，调用 ks 接口，重启所有 client，注册文件。
