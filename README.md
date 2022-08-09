# gecko-mqtt

* 连接 (Connection)、会话 (Session)、路由 (Router)、集群 (Cluster) 分层
* 持久化：会话，配置，路由表

## 集群层

```
              |-------------------|
deviceA <---> | NodeA <---> NodeB | <---> deviceB
              |-------------------|
```
* 对于设备 A，B 来说，集群内部 NodeA 和 NodeB 之间的通信交互是透明的
* 设备A 发送一个 PUBLISH 消息的过程
    * PUBLISH: deviceA -> NodeA -> NodeB -> deviceB
    * PUBACK: deviceB -> NodeB -> NodeA -> deviceA

* Manager
    * 分别提供单机，raft，etcd集群管理器
    * 维护节点信息
* Connection
    * 维护到所有其他节点的 grpc 连接
    * 用于节点间同步信息
* Storage
    * 强一致性存储集群数据
    * 保存路由表，会话等全局共用信息

## 网络层
* Connection
    * 代表一条 tcp 连接
    * tcp read + tcp write
    * 提供 packet 读写
    * **单元测试**
* Packet
    * 报文数据解析
    * bytes <-> packet
    * **单元测试**

## 会话层
* Session 
    * 如果 clea_session = true，会话仅保存到内存即可
    * 如果 clea_session = false，需要集群全局持久化，保存客户端订阅信息
* 处理 Qos0/1/2 消息接收与下发
* 消息超时重传与离线消息保存
* 通过飞行窗口（Inflight Window）实现下发消息吞吐控制与顺序保证

## 协议层
* 依赖网络层 Connection 进行数据读写
* Router
    * 转发层，依赖所有协议报文处理 Handlers
* ConnectionEventLoop
    * 代表一个客户端连接，保存连接信息
    * readloop 从 Protocol 读取报文，提交给 Router
    * keepalive 处理
    * 后台线程阻塞，返回即表示断开连接

## EMQX 架构设计
https://www.emqx.io/docs/zh/v5.0/design/design.html#%E7%B3%BB%E7%BB%9F%E6%9E%B6%E6%9E%84

* 在emqx5.0中，使用 core + replica 模式
* 节点少时，仅使用 core 节点，相当于本项目中的 raft 模式
* 节点多时，使用 core + replica 节点，其中 core 用作存储，replica 用于客户端连接，相当于本项目中的 etcd 模式

## 节点间路由

### 过程
1. 客户端可能连接集群的任一节点
2. publish 消息到达后，需要把消息发送到所有节点，各个节点把消息分发到手下的客户端

### 实现
1. 订阅表: Topic - Client，各个节点维护自己的数据，不需要持久化
1. 路由表: Topic - Node，集群统一维护
2. 接收到 publish 消息的节点，根据路由表，把 publish 消息发送到对应节点
3. 节点间消息转发使用 **GRPC** 实现

### 节点下线/上线
1. 如果节点主动下线，则下线之前删除**路由表**中相关记录
2. 客户端连接连接到新的节点
    * 如果 session 存在，则新节点加载 session 的信息，触发路由表更新操作
    * 如果 session 不存在或不可用，则当 client 重新订阅时，触发路由表更新操作

## TODO
* 自动订阅：设备连接后，集群自动为其订阅默认的主题