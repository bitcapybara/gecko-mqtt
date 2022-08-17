//! 分布层

struct GrpcChannel {}

/// 借助于集群管理器，维护对所有其他对等节点的 tcp 连接
pub(crate) struct Dispatcher {
    // /// 对等节点, key = nodeid
    // conns: HashMap<usize, GrpcChannel>,
}

impl Dispatcher {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
