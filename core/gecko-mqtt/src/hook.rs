use async_trait::async_trait;

use crate::Hook;

pub struct DefaultHook;

impl DefaultHook {
    pub(crate) fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Hook for DefaultHook {
    async fn authenticate(&self) -> bool {
        true
    }

    async fn connected(&self) {}

    async fn disconnect(&self) {}
}
