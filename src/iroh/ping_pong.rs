// use std::io::Read;
// use iroh::{Endpoint};

// pub async fn ping_pong() {
//     // Create an iroh endpoint.
//     let endpoint = Endpoint::builder().discovery_n0().bind().await?;

//     // For the sending side: open a bidirectional stream.
//     let (mut send_stream, mut recv_stream) = endpoint
//         .connect(peer_addr, b"ping-pong")
//         .await?
//         .open_bi()
//         .await?;

//     // In an async loop: on each keyboard event, send its byte representation.
//     tokio::spawn(async move {
//         while let Some(key) = read_from_stdin().await {
//             send_stream.write_all(&[key as u8]).await.expect("Failed to send key");
//         }
//     });

//     // Meanwhile, on the receiving side: read from the stream.
//     tokio::spawn(async move {
//         let mut buf = [0u8; 1];
//         while recv_stream.read_exact(&mut buf).await.is_ok() {
//             process_incoming_key(buf[0]);
//         }
//     });
// }

// async fn read_from_stdin() -> Option<u8> {
//     let mut key = None;
//     let mut buf = [0_u8; 1];
//     let mut stdin = std::io::stdin(); // We get `Stdin` here.
//     loop {
//         stdin.read_exact(&mut buf);
//         key = Some(buf[0]);
//         if key.is_some() { break; }
//     }
//     key
// }

// #[cfg(test)]
// mod tests{
//     use super::*;
//     #[test]
//     #[ignore = "this needs to run in two terminals"]
//     fn run() {
//         ping_pong()
//     }
// }
