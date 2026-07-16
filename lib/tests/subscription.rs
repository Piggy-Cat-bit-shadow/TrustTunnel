use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use trusttunnel::authentication::registry_based::Client;
use trusttunnel::authentication::{registry_based::RegistryBasedAuthenticator, Authenticator};
use trusttunnel::core::Core;
use trusttunnel::settings::{
    Http1Settings, Http2Settings, ListenProtocolSettings, QuicSettings, Settings, TlsHostInfo,
    TlsHostsSettings,
};
use trusttunnel::shutdown::Shutdown;
use trusttunnel::subscription::SubscriptionSettings;

#[allow(dead_code)]
mod common;

fn make_settings(
    listen_address: &SocketAddr,
    subscription_address: &str,
    enabled: bool,
    users: Vec<Client>,
) -> Settings {
    let mut builder = Settings::builder()
        .listen_address(listen_address)
        .unwrap()
        .listen_protocols(ListenProtocolSettings {
            http1: Some(Http1Settings::builder().build()),
            http2: Some(Http2Settings::builder().build()),
            quic: Some(QuicSettings::builder().build()),
        })
        .allow_private_network_connections(true)
        .clients(users);

    if enabled {
        builder = builder.subscription(SubscriptionSettings {
            enabled: true,
            hostname: Some(common::MAIN_DOMAIN_NAME.to_string()),
            path: SubscriptionSettings::default_path(),
            address: Some(subscription_address.to_string()),
            name: Some("Acme Corp VPN".to_string()),
            dns_upstreams: vec!["tls://1.1.1.1".to_string()],
            custom_sni: Some(common::MAIN_DOMAIN_NAME.to_string()),
            client_random_prefix: Some("aabbcc/ff00ff".to_string()),
        });
    }

    builder.build().unwrap()
}

fn make_hosts(cert_path: &str) -> TlsHostsSettings {
    TlsHostsSettings::builder()
        .main_hosts(vec![TlsHostInfo {
            hostname: common::MAIN_DOMAIN_NAME.to_string(),
            cert_chain_path: cert_path.to_string(),
            private_key_path: cert_path.to_string(),
            allowed_sni: vec![],
        }])
        .build()
        .unwrap()
}

fn alice_bob() -> Vec<Client> {
    vec![
        Client {
            username: "alice".to_string(),
            password: "secret".to_string(),
            max_http2_conns: None,
            max_http3_conns: None,
        },
        Client {
            username: "bob".to_string(),
            password: "passwd".to_string(),
            max_http2_conns: None,
            max_http3_conns: None,
        },
    ]
}

// base64("alice:secret") and base64("bob:passwd")
const ALICE_BASIC: &str = "Basic YWxpY2U6c2VjcmV0";
const BOB_BASIC: &str = "Basic Ym9iOnBhc3N3ZA==";

async fn fetch(
    endpoint_address: &SocketAddr,
    auth: Option<&str>,
    extra: &[(&str, &str)],
) -> (http::StatusCode, bytes::Bytes) {
    let stream =
        common::establish_tls_connection(common::MAIN_DOMAIN_NAME, endpoint_address, None).await;
    let url = format!(
        "https://{}:{}/subscription",
        common::MAIN_DOMAIN_NAME,
        endpoint_address.port()
    );
    let mut headers: Vec<(&str, &str)> = Vec::new();
    if let Some(a) = auth {
        headers.push(("authorization", a));
    }
    headers.extend_from_slice(extra);
    let (parts, body) =
        common::do_get_request(stream, http::Version::HTTP_11, &url, &headers).await;
    (parts.status, body)
}

#[tokio::test]
async fn get_with_valid_credentials_returns_200_and_user_fields() {
    common::set_up_logger();
    let endpoint_address = common::make_endpoint_address();
    let cert_file = common::make_cert_key_file();
    let cert_path = cert_file.path.to_str().unwrap();
    let settings = make_settings(&endpoint_address, "203.0.113.1:443", true, alice_bob());
    let hosts = make_hosts(cert_path);

    let client_task = async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let (status, body) = fetch(&endpoint_address, Some(ALICE_BASIC), &[]).await;
        assert_eq!(status, http::StatusCode::OK);
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("\"version\":1"));
        assert!(body.contains("\"hostname\":\"localhost\""));
        assert!(body.contains("\"address\":\"203.0.113.1:443\""));
        assert!(body.contains("\"username\":\"alice\""));
        assert!(body.contains("\"password\":\"secret\""));
        assert!(body.contains("\"name\":\"Acme Corp VPN\""));
        assert!(body.contains("\"dns_upstreams\":[\"tls://1.1.1.1\"]"));
        assert!(body.contains("\"custom_sni\":\"localhost\""));
        assert!(body.contains("\"client_random_prefix\":\"aabbcc/ff00ff\""));
    };

    tokio::select! {
        _ = run_endpoint(settings, hosts) => unreachable!(),
        _ = client_task => (),
        _ = tokio::time::sleep(Duration::from_secs(10)) => panic!("Timed out"),
    }
}

#[tokio::test]
async fn get_without_credentials_returns_401() {
    common::set_up_logger();
    let endpoint_address = common::make_endpoint_address();
    let cert_file = common::make_cert_key_file();
    let settings = make_settings(&endpoint_address, "203.0.113.1:443", true, alice_bob());
    let hosts = make_hosts(cert_file.path.to_str().unwrap());

    let client_task = async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let (status, body) = fetch(&endpoint_address, None, &[]).await;
        assert_eq!(status, http::StatusCode::UNAUTHORIZED);
        assert!(!body.windows(6).any(|w| w == b"secret"));
    };

    tokio::select! {
        _ = run_endpoint(settings, hosts) => unreachable!(),
        _ = client_task => (),
        _ = tokio::time::sleep(Duration::from_secs(10)) => panic!("Timed out"),
    }
}

#[tokio::test]
async fn get_as_bob_returns_bobs_credentials_not_alices() {
    common::set_up_logger();
    let endpoint_address = common::make_endpoint_address();
    let cert_file = common::make_cert_key_file();
    let settings = make_settings(&endpoint_address, "203.0.113.1:443", true, alice_bob());
    let hosts = make_hosts(cert_file.path.to_str().unwrap());

    let client_task = async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let (status, body) = fetch(&endpoint_address, Some(BOB_BASIC), &[]).await;
        assert_eq!(status, http::StatusCode::OK);
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("\"username\":\"bob\""));
        assert!(body.contains("\"password\":\"passwd\""));
        assert!(!body.contains("secret"));
    };

    tokio::select! {
        _ = run_endpoint(settings, hosts) => unreachable!(),
        _ = client_task => (),
        _ = tokio::time::sleep(Duration::from_secs(10)) => panic!("Timed out"),
    }
}

#[tokio::test]
async fn post_returns_405() {
    common::set_up_logger();
    let endpoint_address = common::make_endpoint_address();
    let cert_file = common::make_cert_key_file();
    let settings = make_settings(&endpoint_address, "203.0.113.1:443", true, alice_bob());
    let hosts = make_hosts(cert_file.path.to_str().unwrap());

    let client_task = async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let stream =
            common::establish_tls_connection(common::MAIN_DOMAIN_NAME, &endpoint_address, None)
                .await;
        let url = format!(
            "https://{}:{}/subscription",
            common::MAIN_DOMAIN_NAME,
            endpoint_address.port()
        );
        let response = common::do_post_request(stream, http::Version::HTTP_11, &url, 0).await;
        assert_eq!(response.status(), http::StatusCode::METHOD_NOT_ALLOWED);
    };

    tokio::select! {
        _ = run_endpoint(settings, hosts) => unreachable!(),
        _ = client_task => (),
        _ = tokio::time::sleep(Duration::from_secs(10)) => panic!("Timed out"),
    }
}

#[tokio::test]
async fn disabled_subscription_does_not_serve_data() {
    common::set_up_logger();
    let endpoint_address = common::make_endpoint_address();
    let cert_file = common::make_cert_key_file();
    let settings = make_settings(&endpoint_address, "203.0.113.1:443", false, alice_bob());
    let hosts = make_hosts(cert_file.path.to_str().unwrap());

    let client_task = async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let (status, body) = fetch(&endpoint_address, Some(ALICE_BASIC), &[]).await;
        assert_ne!(status, http::StatusCode::OK);
        assert!(!body.windows(6).any(|w| w == b"secret"));
    };

    tokio::select! {
        _ = run_endpoint(settings, hosts) => unreachable!(),
        _ = client_task => (),
        _ = tokio::time::sleep(Duration::from_secs(10)) => panic!("Timed out"),
    }
}

async fn run_endpoint(settings: Settings, hosts: TlsHostsSettings) {
    let shutdown = Shutdown::new();
    let authenticator: Option<Arc<dyn Authenticator>> = if !settings.get_clients().is_empty() {
        Some(Arc::new(RegistryBasedAuthenticator::new(
            settings.get_clients(),
        )))
    } else {
        None
    };
    let endpoint = Core::new(settings, authenticator, hosts, shutdown).unwrap();
    endpoint.listen().await.unwrap();
}
