use anyhow::Result;

use super::establish_connection;
use crate::iroh::Connection;
use crate::iroh::gossip::Command;

pub async fn join(connection: &mut Connection, ticket: String) -> Result<&mut Connection> {
    connection.command = Command::Join { ticket };
    establish_connection(connection).await?;
    Ok(connection)
}
