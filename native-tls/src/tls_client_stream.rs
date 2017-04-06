// Copyright 2015-2016 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::net::SocketAddr;
use std::io;

use futures::Future;
#[cfg(feature = "mtls")]
use native_tls::Pkcs12;
#[cfg(target_os = "linux")]
use openssl::x509::X509 as OpensslX509;
#[cfg(target_os = "macos")]
use security_framework::certificate::SecCertificate;
use tokio_core::net::TcpStream as TokioTcpStream;
use tokio_core::reactor::Handle;
use tokio_tls::TlsStream as TokioTlsStream;

use trust_dns::BufClientStreamHandle;
use trust_dns::tcp::TcpClientStream;
use trust_dns::client::ClientStreamHandle;

use {TlsStream, TlsStreamBuilder};

pub type TlsClientStream = TcpClientStream<TokioTlsStream<TokioTcpStream>>;

pub struct TlsClientStreamBuilder(TlsStreamBuilder);

impl TlsClientStreamBuilder {
    pub fn new() -> TlsClientStreamBuilder {
        TlsClientStreamBuilder(::tls_stream::tls_builder())
    }

    /// Add a custom trusted peer certificate or certificate auhtority.
    ///
    /// If this is the 'client' then the 'server' must have it associated as it's `identity`, or have had the `identity` signed by this certificate.
    #[cfg(target_os = "macos")]
    pub fn add_ca(&mut self, ca: SecCertificate) {
        self.0.add_ca(ca);
    }

    /// Add a custom trusted peer certificate or certificate auhtority.
    ///
    /// If this is the 'client' then the 'server' must have it associated as it's `identity`, or have had the `identity` signed by this certificate.
    #[cfg(target_os = "linux")]
    pub fn add_ca(&mut self, ca: OpensslX509) {
        self.0.add_ca(ca);
    }

    /// Client side identity for client auth in TLS (aka mutual TLS auth)
    #[cfg(feature = "mtls")]
    pub fn identity(&mut self, pkcs12: Pkcs12) {
        self.0.identity(pkcs12);
    }

    /// Creates a new TlsStream to the specified name_server
    ///
    /// # Arguments
    ///
    /// * `name_server` - IP and Port for the remote DNS resolver
    /// * `subject_name` - The Subject Public Key Info (SPKI) name as associated to a certificate
    /// * `loop_handle` - The reactor Core handle
    pub fn build
        (self,
         name_server: SocketAddr,
         subject_name: String,
         loop_handle: Handle)
         -> (Box<Future<Item = TlsClientStream, Error = io::Error>>, Box<ClientStreamHandle>) {
        let (stream_future, sender) = self.0.build(name_server, subject_name, loop_handle);

        let new_future: Box<Future<Item = TlsClientStream, Error = io::Error>> =
            Box::new(stream_future.map(move |tls_stream| TcpClientStream::from_stream(tls_stream)));

        let sender = Box::new(BufClientStreamHandle::new(name_server, sender));

        (new_future, sender)
    }
}