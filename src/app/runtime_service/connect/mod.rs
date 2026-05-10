use super::*;

mod connect_flow;
mod replace_flow;

impl<'a> RuntimeService<'a> {
    pub fn new(context: &'a AppContext) -> Self {
        Self { context }
    }

    pub async fn disconnect(&self) -> crate::app::Result<DisconnectResult> {
        let stopped_session = stop_active_session(self.context).await?;
        Ok(DisconnectResult { stopped_session })
    }
}
