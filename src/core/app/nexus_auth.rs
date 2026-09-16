// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use serde::Deserialize;

const NEXUS_API_BASE_URL: &str = "https://api.nexusmods.com";
const NEXUS_VALIDATE_PATH: &str = "/v1/users/validate.json";
const NEXUS_SECURE_STORE_SERVICE_NAME: &str = "BIO";
const NEXUS_SECURE_STORE_ACCOUNT_NAME: &str = "nexus-api-key";
const APPLICATION_NAME: &str = "BIO";
const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");
const CONNECT_TIMEOUT_SECS: u64 = 10;
const READ_TIMEOUT_SECS: u64 = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NexusAccount {
    pub name: String,
    pub is_premium: bool,
}

pub(crate) type NexusLoginResult = Result<NexusAccount, String>;

#[derive(Debug, Deserialize)]
struct NexusValidateResponse {
    #[serde(default)]
    name: String,
    #[serde(default)]
    is_premium: bool,
}

#[must_use]
pub fn nexus_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .timeout_read(Duration::from_secs(READ_TIMEOUT_SECS))
        .build()
}

pub(crate) fn nexus_get(
    agent: &ureq::Agent,
    base_url: &str,
    path: &str,
    api_key: &str,
) -> ureq::Request {
    agent
        .get(&format!("{base_url}{path}"))
        .set("apikey", api_key)
        .set("Application-Name", APPLICATION_NAME)
        .set("Application-Version", APPLICATION_VERSION)
        .set("User-Agent", &format!("BIO/{APPLICATION_VERSION}"))
        .set("Accept", "application/json")
}

pub(crate) fn validate_api_key_at(
    agent: &ureq::Agent,
    base_url: &str,
    api_key: &str,
) -> NexusLoginResult {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err("Paste your personal API key first.".to_string());
    }
    let response = match nexus_get(agent, base_url, NEXUS_VALIDATE_PATH, api_key).call() {
        Ok(response) => response,
        Err(ureq::Error::Status(401, _)) => {
            return Err(
                "Nexus Mods rejected this key. Copy it again from the API keys page.".to_string(),
            );
        }
        Err(ureq::Error::Status(code, _)) => {
            return Err(format!("Nexus Mods returned HTTP {code}."));
        }
        Err(ureq::Error::Transport(transport)) => {
            return Err(format!("Could not reach Nexus Mods: {transport}"));
        }
    };
    let text = match response.into_string() {
        Ok(text) => text,
        Err(err) => return Err(format!("Nexus Mods returned an unexpected reply: {err}")),
    };
    let parsed = match serde_json::from_str::<NexusValidateResponse>(&text) {
        Ok(parsed) => parsed,
        Err(err) => return Err(format!("Nexus Mods returned an unexpected reply: {err}")),
    };
    let name = parsed.name.trim();
    if name.is_empty() {
        return Err("Nexus Mods accepted the key but returned no account name.".to_string());
    }
    Ok(NexusAccount {
        name: name.to_string(),
        is_premium: parsed.is_premium,
    })
}

pub(crate) fn validate_api_key(api_key: &str) -> NexusLoginResult {
    validate_api_key_at(&nexus_agent(), NEXUS_API_BASE_URL, api_key)
}

pub(crate) fn start_nexus_key_validation(api_key: String) -> Receiver<NexusLoginResult> {
    let (tx, rx) = mpsc::channel::<NexusLoginResult>();
    thread::spawn(move || {
        let result = match validate_api_key(&api_key) {
            Ok(account) => match store_nexus_api_key(&api_key) {
                Ok(()) => Ok(account),
                Err(err) => Err(format!("Could not store the key: {err}")),
            },
            Err(err) => Err(err),
        };
        let _ = tx.send(result);
    });
    rx
}

pub(crate) fn poll_nexus_key_validation(
    rx: &mut Option<Receiver<NexusLoginResult>>,
) -> Option<NexusLoginResult> {
    let receiver = rx.as_ref()?;
    match receiver.try_recv() {
        Ok(result) => {
            *rx = None;
            Some(result)
        }
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => {
            *rx = None;
            Some(Err(
                "The Nexus Mods check stopped before it finished.".to_string()
            ))
        }
    }
}

pub(crate) fn load_nexus_account_from_stored_key() -> Result<Option<NexusAccount>, String> {
    let Some(key) = load_nexus_api_key()? else {
        return Ok(None);
    };
    validate_api_key(&key).map(Some)
}

pub(crate) fn load_nexus_api_key() -> Result<Option<String>, String> {
    match secure_store_entry()?.get_password() {
        Ok(key) => {
            let key = key.trim();
            if key.is_empty() {
                Ok(None)
            } else {
                Ok(Some(key.to_string()))
            }
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err.to_string()),
    }
}

pub(crate) fn store_nexus_api_key(api_key: &str) -> Result<(), String> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err("nexus api key is empty".to_string());
    }
    secure_store_entry()?
        .set_password(api_key)
        .map_err(|err| err.to_string())
}

pub(crate) fn clear_nexus_api_key() -> Result<(), String> {
    match secure_store_entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

fn secure_store_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(
        NEXUS_SECURE_STORE_SERVICE_NAME,
        NEXUS_SECURE_STORE_ACCOUNT_NAME,
    )
    .map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::Instant;

    fn spawn_fixture(status_line: &str, body: &'static str) -> (String, Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let base = format!("http://{addr}");
        let status_line = status_line.to_string();
        let (tx, rx) = mpsc::channel::<String>();
        thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request_text = String::from_utf8_lossy(&buf[..n]).to_string();
            let response = format!(
                "{status_line}\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
            let _ = tx.send(request_text);
        });
        (base, rx)
    }

    #[test]
    fn validate_parses_a_premium_account_and_sends_the_required_headers() {
        let (base, rx) = spawn_fixture(
            "HTTP/1.1 200 OK",
            r#"{"user_id":7,"key":"k","name":"Xgatt","is_premium":true,"is_supporter":false,"email":"","profile_url":""}"#,
        );
        let agent = nexus_agent();
        let result = validate_api_key_at(&agent, &base, "secret-key");
        assert_eq!(
            result,
            Ok(NexusAccount {
                name: "Xgatt".to_string(),
                is_premium: true,
            })
        );
        let request_text = rx.recv().unwrap().to_lowercase();
        assert!(request_text.contains("get /v1/users/validate.json"));
        assert!(request_text.contains("apikey: secret-key"));
        assert!(request_text.contains("application-name: bio"));
        assert!(request_text.contains(&format!(
            "application-version: {}",
            env!("CARGO_PKG_VERSION")
        )));
        assert!(request_text.contains("user-agent: bio/"));
    }

    #[test]
    fn validate_reads_a_free_account() {
        let (base, _rx) = spawn_fixture(
            "HTTP/1.1 200 OK",
            r#"{"user_id":7,"key":"k","name":"Xgatt","is_premium":false,"is_supporter":false,"email":"","profile_url":""}"#,
        );
        let agent = nexus_agent();
        let result = validate_api_key_at(&agent, &base, "secret-key").unwrap();
        assert!(!result.is_premium);
    }

    #[test]
    fn validate_maps_401_to_the_rejected_message() {
        let (base, _rx) = spawn_fixture("HTTP/1.1 401 Unauthorized", "");
        let agent = nexus_agent();
        let result = validate_api_key_at(&agent, &base, "secret-key");
        assert_eq!(
            result,
            Err("Nexus Mods rejected this key. Copy it again from the API keys page.".to_string())
        );
    }

    #[test]
    fn validate_refuses_a_blank_key_without_a_request() {
        let agent = nexus_agent();
        let started = Instant::now();
        let result = validate_api_key_at(&agent, "http://127.0.0.1:1", "   ");
        assert_eq!(
            result,
            Err("Paste your personal API key first.".to_string())
        );
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn validate_rejects_a_reply_without_a_name() {
        let (base, _rx) = spawn_fixture("HTTP/1.1 200 OK", r#"{"is_premium":true}"#);
        let agent = nexus_agent();
        let result = validate_api_key_at(&agent, &base, "secret-key");
        assert_eq!(
            result,
            Err("Nexus Mods accepted the key but returned no account name.".to_string())
        );
    }

    #[test]
    fn poll_returns_nothing_while_the_worker_runs_and_clears_the_channel_on_a_result() {
        let (tx, rx) = mpsc::channel::<NexusLoginResult>();
        let mut rx = Some(rx);
        assert_eq!(poll_nexus_key_validation(&mut rx), None);
        tx.send(Ok(NexusAccount {
            name: "Xgatt".to_string(),
            is_premium: true,
        }))
        .unwrap();
        let result = poll_nexus_key_validation(&mut rx);
        assert_eq!(
            result,
            Some(Ok(NexusAccount {
                name: "Xgatt".to_string(),
                is_premium: true,
            }))
        );
        assert!(rx.is_none());
    }

    #[test]
    fn poll_reports_a_dropped_worker() {
        let (tx, rx) = mpsc::channel::<NexusLoginResult>();
        let mut rx = Some(rx);
        drop(tx);
        let result = poll_nexus_key_validation(&mut rx);
        assert_eq!(
            result,
            Some(Err(
                "The Nexus Mods check stopped before it finished.".to_string()
            ))
        );
        assert!(rx.is_none());
    }
}
