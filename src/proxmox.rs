use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct VncTicketResponse {
    pub data: VncTicketData,
}

#[derive(Deserialize, Debug)]
pub struct VncTicketData {
    pub ticket: String,
    pub port: String,
}

pub async fn get_vnc_ticket(
    url: &str,
    token_id: &str,
    token_secret: &str,
    node: &str,
    vmid: &str,
) -> Result<VncTicketData, String> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true) // Ignore self-signed Proxmox certs
        .build().map_err(|e| e.to_string())?;

    let endpoint = format!("{}/nodes/{}/qemu/{}/vncproxy", url, node, vmid);
    let auth_header = format!("PVEAPIToken={}={}", token_id, token_secret);

    let res = client
        .post(&endpoint)
        .header("Authorization", auth_header)
        .send()
        .await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Proxmox API Error: {}", res.status()));
    }

    let ticket_res: VncTicketResponse = res.json().await.map_err(|e| e.to_string())?;
    Ok(ticket_res.data)
}
