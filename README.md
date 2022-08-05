# gecko-mqtt

## 网络层
* Network
    * 依赖 Connection
    * 后台 readloop + writeloop
    * 提供读写 bytes 的方法
* Connection
    * tcp read + tcp write
    * **单元测试**

## 协议层
* 依赖网络层 Network 进行数据读写
* Packet
    * 报文数据解析
    * bytes <-> packet
    * **单元测试**
* Protocol
    * Network + Packet
    * 以 Packet 为单位进行数据读写
* Router
    * 转发层，依赖所有协议报文处理 Handlers
* Connection
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
