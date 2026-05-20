# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [1.0.41]

### Added

- `non_connect_auth_failure_status_code` config key to control the HTTP status code returned on authentication failure for non-CONNECT requests. Extend allowed values for `auth_failure_status_code` and `non_connect_auth_failure_status_code` to 407, 405, 404, and 403.

## [1.0.28]

### Added

- `trusttunnel_endpoint -c` can now generate `client_random_prefix` values automatically, append matching allow rules to `rules.toml`, and embed the generated value into exported client configs.

## [1.0.17]

### Added

- `ping_enable`, `ping_path`, `speedtest_enable` and `speedtest_path` config keys to configure ping and speedtest handlers.
- `auth_failure_status_code` config key to control the HTTP status code returned on authentication failure (407 or 405). Defaults to 407.

### Fixed

- Reverse proxy routing for H2/H3.

## [1.0.16]

### Fixed

- HTTP/1.1 codec busy loop when receiving partial request headers.

## [1.0.13]

### Fixed

- Change deep-link format from `tt://` to `tt://?`. For backward compatibility, `tt://` is still supported.

## [1.0.11]

### Security

- Fixed traffic leaking to local network via UDP, ICMP, and SOCKS5 forwarders
  when `allow_private_network_connections` is set to `false`.
    - Added `is_global_ip` check to UDP forwarder
    - Added `is_global_ip` check to ICMP forwarder
    - Added `is_global_ip` check to SOCKS5 forwarder (TCP and UDP)
    - Handle IPv4-mapped IPv6 addresses (`::ffff:x.x.x.x`) in `is_global_ip`
  (Based on [GitHub PR #79](https://github.com/TrustTunnel/TrustTunnel/pull/79) by @andrew-morris)

## [1.0.7]

### Added

- Per-client connection limits
    - Optional limits for simultaneous HTTP/2 and HTTP/3 connections per client credentials
    - Global default limits via `default_max_http2_conns_per_client` and `default_max_http3_conns_per_client` in main config
    - Per-client overrides via `max_http2_conns` and `max_http3_conns` in credentials file
    - Applies to both SNI-authenticated and proxy-basic authenticated connections
    - For proxy-basic: limit enforced on first authenticated request (not idle connections)

  API changes in the library:
    - Added `max_http2_conns` and `max_http3_conns` fields to `authentication::registry_based::Client`
    - Added `default_max_http2_conns_per_client` and `default_max_http3_conns_per_client` fields to `settings::Settings`
    - Added new `connection_limiter` module with `ConnectionLimiter` and `ConnectionGuard` types
    - Added `connection_limiter` field to `core::Context`

## [1.0.6]

### Added

- Support for X25519MLKEM768 post-quantum group.

## [1.0.5]

### Added

- The `-a` flag now accepts `domain` and `domain:port` in addition to `ip` and `ip:port`.
  The exported client configuration will contain the domain name, which the client resolves via DNS at connect time.
- Deep-link format (`tt://`) now supports domain names in the `addresses` field.
- When listening on `[::]`, the endpoint now explicitly sets `IPV6_V6ONLY=false` to accept
  both IPv4 and IPv6 connections on a single socket (dual-stack).

## [1.0.1]

### Added

- New `trusttunnel-deeplink` library crate for encoding/decoding `tt://` URIs
- `client_random_prefix` field to client configuration export
    - New CLI option `--client-random-prefix`
    - Validates hex format and checks against `rules.toml`
    - Added to deep-link format as tag 0x0B

## [0.9.127]

### Added

- GPG signing of the endpoint binaries.

## [0.9.122]

### Changed

- Endpoint now requires credentials when listening on a public address.
- Added support of shortened QUIC settings names in configuration files.

## [0.9.115]

### Security

- Fixed an issue where `client_random_prefix` rules didn't match when Anti-DPI or post-quantum cryptography was enabled.
  (<https://github.com/TrustTunnel/TrustTunnel/security/advisories/GHSA-fqh7-r5gf-3r87>)

## [0.9.114]

### Security

- Fixed an issue where `allow_private_network_connections` set to false could be bypassed
  when a numeric address was used.
  (<https://github.com/TrustTunnel/TrustTunnel/security/advisories/GHSA-hgr9-frvw-5r76>)

## [0.9.87]

### Added

- Automatic Let's Encrypt certificate generation to `setup_wizard`
- [CONFIGURATION.md](CONFIGURATION.md)
- Improved the CLI interface of `setup_wizard` and provided better post-setup
  guidance there.

## [0.9.77]

### Added

- Install script for the endpoint
- Linter scripts and reformatted the code accordingly

### Changed

- Structure of the `scripts` folder

### Fixed

- Project warnings

## [0.9.61]

### Changed

- Removed old docker image
- Added new [docker image](Dockerfile) with improved build and run logic

## [0.9.56]

### Added

- A [docker image](docker/Dockerfile) with a configured and running endpoint.
- A [Makefile](Makefile) to simplify building and running the endpoint.
- Setup Wizard now doesn't ask for parameters specified through command line arguments.
  E.g., with `setup_wizard --lib-settings vpn.toml` it won't ask a user for the library
  settings file path.

## [0.9.47]

### Removed

- RADIUS-based authenticator

## [0.9.45]

### Changed

- The executable now expects that the configuration files are TOML-formatted

## [0.9.38]

### Fixed

- Enormous timeout of TCP connections establishment procedure.
  API changes in the library:
    - added `connection_establishment_timeout` field into `settings::Settings`

  The executable related changes:
    - the settings file is changed accordingly to the changes described above

## [0.9.36]

### Changed

- The endpoint is now capable of handling service requests on the main tls domain.
  API changes in the library:
    - `tunnel_hosts` field of `settings::TlsHostsSettings` structure is renamed to `main_hosts`
    - `path_mask` field added into `settings::ReverseProxySettings`

  The executable related changes:
    - the settings file is changed accordingly to the changes described above

## [0.9.30]

### Added

- Support for dynamic reloading of TLS hosts settings.
  API changes in the library:
    - `tunnel_tls_hosts`, `ping_tls_hosts` and `speed_tls_hosts` from `settings::Settings`,
      and `tls_hosts` from `settings::ReverseProxySettings` were extracted into a dedicated
      structure `settings::TlsHostsSettings`
    - Added a new method for the reloading settings: `core::Core::reload_tls_hosts_settings()`

  The executable related changes:
    - The TLS hosts settings must be passed as a separate argument ([see here](./README.md#running) for details)
    - The new settings file structures are described ([see here](./README.md#library-configuration))
    - The executable is now handling the SIGHUP signal to trigger the reloading
      ([see here](./README.md#dynamic-reloading-of-tls-hosts-settings) for details)

## [0.9.29]

### Changed

- Removed blocking `core::Core::listen()` method. The library user must now set up a tokio runtime itself.
  The library API changes:
    - Removed `core::Core::listen()`
    - `core::Core::listen_async()` renamed to `core::Core::listen()`
    - Removed `threads_number` field from `settings::Settings`

  The executable related changes:
    - `threads_number` field in a settings file is now ignored
    - The number of worker threads may be specified via commandline argument (see the executable help for details)

## [0.9.28]

### Changed

- Added support for configuring the library with multiple TLS certificates.
  API changes:
    - `settings::Settings::tunnel_tls_host_info` is renamed to `settings::Settings::tunnel_tls_hosts` and is now a vector of hosts
    - `settings::Settings::ping_tls_host_info` is renamed to `settings::Settings::ping_tls_hosts` and is now a vector of hosts
    - `settings::Settings::speed_tls_host_info` is renamed to `settings::Settings::speed_tls_hosts` and is now a vector of hosts
    - `settings::ReverseProxySettings::tls_host_info` is renamed to `settings::ReverseProxySettings::tls_hosts` and is now a vector of hosts

## [0.9.24]

### Added

- Speedtest support

## [0.9.13]

### Changed

- Test changelog entry please ignore

[Unreleased]: https://github.com/TrustTunnel/TrustTunnel/compare/v1.0.41...HEAD
[1.0.41]: https://github.com/TrustTunnel/TrustTunnel/compare/v1.0.28...v1.0.41
[1.0.28]: https://github.com/TrustTunnel/TrustTunnel/compare/v1.0.17...v1.0.28
[1.0.17]: https://github.com/TrustTunnel/TrustTunnel/compare/v1.0.16...v1.0.17
[1.0.16]: https://github.com/TrustTunnel/TrustTunnel/compare/v1.0.13...v1.0.16
[1.0.13]: https://github.com/TrustTunnel/TrustTunnel/compare/v1.0.11...v1.0.13
[1.0.11]: https://github.com/TrustTunnel/TrustTunnel/compare/v1.0.7...v1.0.11
[1.0.7]: https://github.com/TrustTunnel/TrustTunnel/compare/v1.0.6...v1.0.7
[1.0.6]: https://github.com/TrustTunnel/TrustTunnel/compare/v1.0.5...v1.0.6
[1.0.5]: https://github.com/TrustTunnel/TrustTunnel/compare/v1.0.1...v1.0.5
[1.0.1]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.127...v1.0.1
[0.9.127]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.122...v0.9.127
[0.9.122]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.115...v0.9.122
[0.9.115]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.114...v0.9.115
[0.9.114]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.87...v0.9.114
[0.9.87]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.77...v0.9.87
[0.9.77]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.61...v0.9.77
[0.9.61]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.56...v0.9.61
[0.9.56]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.47...v0.9.56
[0.9.47]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.45...v0.9.47
[0.9.45]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.38...v0.9.45
[0.9.38]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.36...v0.9.38
[0.9.36]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.30...v0.9.36
[0.9.30]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.29...v0.9.30
[0.9.29]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.28...v0.9.29
[0.9.28]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.24...v0.9.28
[0.9.24]: https://github.com/TrustTunnel/TrustTunnel/compare/v0.9.13...v0.9.24
[0.9.13]: https://github.com/TrustTunnel/TrustTunnel/releases/tag/v0.9.13
