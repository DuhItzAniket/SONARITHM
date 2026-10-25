// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The network-deny harness's negative test (B01-T06): `cargo xtask network-deny
//! --self-test` runs this probe and fails unless the OS mechanism sees its connections.
//! It is ignored in normal test runs, which must make no connection at all.

use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::time::Duration;

/// TEST-NET-1 (RFC 5737): reserved for documentation, routed nowhere.
const NOWHERE: &str = "192.0.2.1:9";

#[test]
#[ignore = "connects on purpose; run only by `cargo xtask network-deny --self-test`"]
fn network_probe_connects() {
    let Ok(address) = NOWHERE.parse::<SocketAddr>() else {
        return;
    };
    // Both attempts are expected to fail (or be blocked); only the attempt matters. Each
    // outcome is printed, so the harness can tell a sandbox's refusal (EPERM) from a
    // timeout when the OS log is late (macOS).
    let tcp = TcpStream::connect_timeout(&address, Duration::from_millis(500)).err();
    println!("network-probe tcp: {}", outcome(tcp.as_ref()));
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        let udp = socket
            .send_to(b"sonarithm network-deny probe", address)
            .err();
        println!("network-probe udp: {}", outcome(udp.as_ref()));
    }
}

/// `sent`, or the OS error number, such as `os error 1` for EPERM.
fn outcome(err: Option<&std::io::Error>) -> String {
    match err {
        None => "sent".to_owned(),
        Some(err) => err.raw_os_error().map_or_else(
            || format!("{:?}", err.kind()),
            |code| format!("os error {code}"),
        ),
    }
}
