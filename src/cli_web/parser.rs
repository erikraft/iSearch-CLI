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
#[serde(untagged)]
enum VersionValue {
    Text(String),
    Number(u64),
}

impl VersionValue {
    fn into_string(self) -> String {
        match self {
            Self::Text(value) => value,
            Self::Number(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FeatureValue {
    List(Vec<String>),
    Map(BTreeMap<String, serde_json::Value>),
}

impl FeatureValue {
    fn into_names(self) -> Vec<String> {
        match self {
            Self::List(values) => values,
            Self::Map(values) => values.into_keys().collect(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct ClientMetadata {
    cli: Option<bool>,
    name: Option<String>,
    #[serde(rename = "clientType")]
    client_type: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ClientIdentification {
    #[serde(rename = "clientType")]
    client_type: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ProtocolMetadata {
    websocket: Option<String>,
    #[serde(rename = "signalingServer")]
    signaling_server: Option<String>,
    #[serde(rename = "wsUrl")]
    ws_url: Option<String>,
    #[serde(rename = "clientIdentification")]
    client_identification: Option<ClientIdentification>,
    messages: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct CliApiResponse {
    cli: Option<bool>,
    version: Option<VersionValue>,
    #[serde(rename = "endpoint")]
    _endpoint: Option<String>,
    #[serde(rename = "clientType")]
    client_type: Option<String>,
    #[serde(rename = "signalingServer")]
    signaling_server: Option<String>,
    #[serde(rename = "wsUrl")]
    ws_url: Option<String>,
    features: Option<FeatureValue>,
    message: Option<String>,
    title: Option<String>,
    description: Option<String>,
    client: Option<ClientMetadata>,
    protocol: Option<ProtocolMetadata>,
}

impl CliApiResponse {
    fn client_type(&self) -> Option<String> {
        self.client_type
            .clone()
            .or_else(|| self.client.as_ref().and_then(|client| client.client_type.clone()))
            .or_else(|| {
                self.protocol.as_ref()
                    .and_then(|protocol| protocol.client_identification.as_ref())
                    .and_then(|identification| identification.client_type.clone())
            })
    }

    fn ws_url(&self) -> Option<String> {
        self.ws_url.clone().or_else(|| {
            self.protocol.as_ref().and_then(|protocol| protocol.websocket.clone())
        }).or_else(|| {
            self.protocol.as_ref().and_then(|protocol| protocol.ws_url.clone())
        })
    }

    fn features(&self) -> Vec<String> {
        if let Some(features) = &self.features {
            return match features {
                FeatureValue::List(values) => values.clone(),
                FeatureValue::Map(values) => values.keys().cloned().collect(),
            };
        }

        self.protocol.as_ref()
            .and_then(|protocol| protocol.messages.clone())
            .unwrap_or_default()
    }
}

/// Parses both the original Drop CLI metadata response and the public v1-compatible shape.
pub fn parse_cli_json(json: &str) -> Result<CliDocument, Box<dyn std::error::Error>> {
    let response: CliApiResponse = serde_json::from_str(json)?;

    if response.cli == Some(false)
        || response.client.as_ref().and_then(|client| client.cli) == Some(false)
    {
        return Err("Drop API returned a non-CLI response".into());
    }

    let server_version = response.version.map(VersionValue::into_string);
    let title = response.title
        .or_else(|| response.client.as_ref().and_then(|client| client.name.clone()))
        .unwrap_or_else(|| "ErikrafT Drop CLI".to_string());
    let message = response.message.or(response.description);
    let signaling_server = response.signaling_server.or_else(|| {
        response.protocol.as_ref().and_then(|protocol| protocol.signaling_server.clone())
    });

    Ok(CliDocument {
        title,
        message,
        client_type: response.client_type(),
        server_version,
        signaling_server,
        ws_url: response.ws_url(),
        features: response.features(),
        raw_json: json.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_current_drop_api_cli_response() {
        let json = r#"{
            "version": 1,
            "client": {
                "name": "ErikrafT Drop",
                "cli": true,
                "officialClient": "iSearch CLI"
            },
            "title": "ErikrafT Drop",
            "description": "Fast peer-to-peer file transfer.",
            "features": ["Send files", "Receive files", "Device discovery"],
            "protocol": {
                "websocket": "/server?client_type=isearch-cli&webrtc_supported=false",
                "clientIdentification": { "clientType": "isearch-cli" },
                "messages": ["ws-config", "peer-joined", "progress"]
            },
            "security": {
                "executeReceivedFiles": false,
                "executeReceivedCode": false,
                "secretsInResponse": false
            }
        }"#;

        let document = parse_cli_json(json).expect("current API response should parse");
        assert_eq!(document.server_version.as_deref(), Some("1"));
        assert_eq!(document.client_type.as_deref(), Some("isearch-cli"));
        assert_eq!(
            document.ws_url.as_deref(),
            Some("/server?client_type=isearch-cli&webrtc_supported=false")
        );
        assert_eq!(document.features, vec!["Send files", "Receive files", "Device discovery"]);
        assert_eq!(document.title, "ErikrafT Drop");
    }

    #[test]
    fn parses_legacy_object_feature_shape() {
        let json = r#"{
            "cli": true,
            "version": "1",
            "clientType": "isearch-cli",
            "signalingServer": "drop.erikraft.com",
            "wsUrl": "wss://drop.erikraft.com/server",
            "features": { "deviceDiscovery": true, "pairing": true }
        }"#;

        let document = parse_cli_json(json).expect("legacy response should parse");
        assert_eq!(document.client_type.as_deref(), Some("isearch-cli"));
        assert_eq!(document.features, vec!["deviceDiscovery", "pairing"]);
    }

    #[test]
    fn rejects_explicit_non_cli_response() {
        assert!(parse_cli_json(r#"{"cli": false}"#).is_err());
    }
}
