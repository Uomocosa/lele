use anyhow::Result;

use super::establish_connection;
use crate::iroh::Connection;
use crate::iroh::gossip::Command;

pub async fn open(connection: &mut Connection) -> Result<&mut Connection> {
    let topic = connection.topic;
    connection.command = Command::Open { topic };
    establish_connection(connection).await?;
    Ok(connection)
}
