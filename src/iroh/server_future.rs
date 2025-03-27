use anyhow::Result;

use super::Server;

#[derive(Debug)]
pub struct ServerFuture {
    pub handle: tokio::task::JoinHandle<anyhow::Result<Server>>,
}

impl ServerFuture {
    pub async fn close(self) -> Result<()> {
        if self.handle.is_finished() {
            let server = self.handle.await??;
            server.close().await?;
        } else  {
            self.handle.abort();
        }
        Ok(())
    }
}