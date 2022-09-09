use self::{candidate::Candidate, follower::Follower, leader::Leader};

pub mod candidate;
pub mod follower;
pub mod leader;

pub enum RoleState {
    Leader(Role<Leader>),
    Follower(Role<Follower>),
    Candidate(Role<Candidate>),
}

pub struct Role<T> {
    role: T,
}
