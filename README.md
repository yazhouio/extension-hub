# plugin-hub

分为服务端和客户端，分别负责文件的发送和接收。

大体流程如图。
![image](https://github.com/yazhouio/plugin-hub/assets/17949154/e1b30011-6bcb-430f-8d08-3c365bb22ec5)


客户端和服务端通信均使用 grpc。

为方便调试，服务端额外提供 http 服务器，负责提供表单上传功能
