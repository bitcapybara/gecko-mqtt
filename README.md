# gecko-mqtt

## 网络层
* Connection
    * 代表一条 tcp 连接
    * tcp read + tcp write
    * 提供 packet 读写
    * **单元测试**

## 协议层
* 依赖网络层 Connection 进行数据读写
* Packet
    * 报文数据解析
    * bytes <-> packet
    * **单元测试**
* Router
    * 转发层，依赖所有协议报文处理 Handlers
* ConnectionEventLoop
    * 代表一个客户端连接，保存连接信息
    * readloop 从 Protocol 读取报文，提交给 Router
    * keepalive 处理
    * 后台线程阻塞，返回即表示断开连接
* Session
    * 一个客户端会话
    * 生命周期是否与 Connection 相同取决于 clean session 配置
* Handlers
    * 可能依赖 Session
    * 向 Protocol 写入报文数据

## EMQX 架构设计
https://www.emqx.io/docs/zh/v5.0/design/design.html#%E7%B3%BB%E7%BB%9F%E6%9E%B6%E6%9E%84

## 节点间路由

### 过程
1. 客户端可能连接集群的任一节点
2. publish 消息到达后，需要把消息发送到所有节点，各个节点把消息分发到手下的客户端

### 实现
1. 订阅表: 主题 - 订阅者
2. 接收到 publish 消息的节点，根据路由表，把 publish 消息发送到对应节点
3. 节点间消息转发使用 **GRPC** 实现