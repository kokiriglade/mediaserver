use std::str::FromStr;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct WebServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(with = "url_serde")]
    pub listen_url: Url,
    #[serde(with = "url_serde")]
    pub redirect_index_to: Url,
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 3000,
            listen_url: Url::from_str("http://localhost:3000")
                .expect("default 'url' should be parseable"),
            redirect_index_to: Url::from_str(
                "http://github.com/kokiriglade/mediaserver",
            )
            .expect("default 'redirect_index_to' should be parseable"),
        }
    }
}
