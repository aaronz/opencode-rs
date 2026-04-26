#![allow(dead_code)]

use crate::{AcpClient, AcpError};

pub async fn cmd_status(client: &AcpClient) -> Result<(), AcpError> {
    let status = client.status().await?;
    println!(
        "ACP Status: {}",
        if status.connected {
            "connected"
        } else {
            "disconnected"
        }
    );
    if let Some(id) = &status.client_id {
        println!("Client ID: {}", id);
    }
    if !status.capabilities.is_empty() {
        println!("Capabilities: {}", status.capabilities.join(", "));
    }
    Ok(())
}

pub async fn cmd_connect(
    client: &AcpClient,
    url: &str,
    client_id: Option<&str>,
) -> Result<(), AcpError> {
    let cid = client_id.map(|s| s.to_string());
    client.connect(url, cid).await?;
    println!("Connected to {}", url);
    Ok(())
}

pub async fn cmd_ack(
    client: &AcpClient,
    handshake_id: &str,
    accepted: bool,
) -> Result<(), AcpError> {
    client.ack(handshake_id, accepted).await?;
    println!("Handshake acknowledgement sent");
    Ok(())
}

pub async fn cmd_handshake(
    client: &AcpClient,
    server_url: &str,
    client_id: &str,
    capabilities: &[String],
) -> Result<(), AcpError> {
    let response = client
        .handshake(server_url, client_id.to_string(), capabilities.to_vec())
        .await?;
    println!("Server ID: {}", response.server_id);
    println!(
        "Accepted capabilities: {}",
        response.accepted_capabilities.join(", ")
    );
    if let Some(token) = &response.session_token {
        println!("Session token: {}", token);
    }
    Ok(())
}
