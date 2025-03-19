use iroh::RelayMode;

pub fn fmt_relay_mode(relay_mode: &RelayMode) -> String {
    match relay_mode {
        RelayMode::Disabled => "None".to_string(),
        RelayMode::Default => "Default Relay (production) servers".to_string(),
        RelayMode::Staging => "Default Relay (staging) servers".to_string(),
        RelayMode::Custom(map) => map
            .urls()
            .map(|url| url.to_string())
            .collect::<Vec<_>>()
            .join(", "),
    }
}
