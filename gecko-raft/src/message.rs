use crate::{MessageId, NodeId, Term};

/// 节点间发送消息携带的元数据
pub struct Message {
    id: MessageId,
    term: Term,
    from: NodeId,
    to: NodeId,
    event: Event,
}

/// 1. 当前节点客户端命令，针对应用状态机的操作
/// 2. 远程节点之间，是针对 raft 状态机的操作
pub enum Event {
    Command(Command),
    CommandReply(CommandReply),
    AppendEntries(AppendEntries),
    AppendEntriesReply(AppendEntriesReply),
    RequestVote(RequestVote),
    RequestVoteReply(RequestVoteReply),
    InstallSnapshot(InstallSnapshot),
    InstallSnapshotReply(InstallSnapshotReply),
    /// 本地状态机生成的快照数据
    Serialized(Vec<u8>),

    Apply(Vec<u8>),
    Serialize,
    Install(Vec<u8>),
}

/// 当前节点客户端请求
pub enum Command {
    Read(Vec<u8>),
    Write(Vec<u8>),
    Status,
}

/// 当前节点客户端回复
pub enum CommandReply {
    Data(Vec<u8>),
    Status(Status),
}

/// raft 当前状态
pub struct Status {}

pub struct AppendEntries {}

pub struct AppendEntriesReply {}

pub struct RequestVote {}

pub struct RequestVoteReply {}

pub struct InstallSnapshot {}

pub struct InstallSnapshotReply {}

/// 消息的来源
pub enum FromAddress {
    /// 来自当前节点的请求（读写状态机）
    Local,
    /// 来自对等节点的请求
    Peer(NodeId),
}

/// 消息的去向
pub enum ToAddress {
    /// 对当前节点的回复
    Local,
    /// 发送给所有节点
    Peers,
}
