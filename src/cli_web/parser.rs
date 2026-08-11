use serde::Deserialize;
use std::collections::BTreeMap;

/// Structured CLI document data retrieved from the Drop API.
#[derive(Debug, Clone)]
pub struct CliDocument {
    pub title: String,
    pub message: Option<String>,
    pub client_type: Option<String>,
    pub server_version: Option<String>,
    pub signaling_server: Option<String>,
    pub ws_url: Option<String>,
    pub features: Vec<String>,
    pub raw_json: String,
}

#[derive(Debug, Deserialize)]
struct CliApiResponse {
    cli: Option<bool>,
    version: Option<String>,
    #[serde(rename = "endpoint")]
    _endpoint: Option<String>,
    #[serde(rename = "clientType")]
    client_type: Option<String>,
    #[serde(rename = "signalingServer")]
    signaling_server: Option<String>,
    #[serde(rename = "wsUrl")]
    ws_url: Option<String>,
    features: Option<BTreeMap<String, serde_json::Value>>,
    message: Option<String>,
}

/// Parses a Drop CLI API response into a CLI document model.
pub fn parse_cli_json(json: &str) -> Result<CliDocument, Box<dyn std::error::Error>> {
    let response: CliApiResponse = serde_json::from_str(json)?;

    if response.cli == Some(false) {
        return Err("Drop API returned a non-CLI response".into());
    }

    let features = response
        .features
        .map(|map| map.into_keys().collect())
        .unwrap_or_default();

    Ok(CliDocument {
        title: "ErikrafT Drop CLI".to_string(),
        message: response.message,
        client_type: response.client_type,
        server_version: response.version,
        signaling_server: response.signaling_server,
        ws_url: response.ws_url,
        features,
        raw_json: json.to_string(),
    })
}
