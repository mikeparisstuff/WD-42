#![crate_id = "rustic_server"]

extern crate time;
extern crate http;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::Writer;

use http::server::{Config, Server, Request, ResponseWriter};
use http::server::request::{Star, AbsoluteUri, AbsolutePath, Authority};
use http::status::{BadRequest, MethodNotAllowed};
use http::method::{Get, Head, Post, Put, Delete, Trace, Options, Connect, Patch};
use http::headers::content_type::MediaType;

impl Server for App {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: self.port }}
    }

    fn handle_request(&self, r: &Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());
        w.headers.server = Some(~"Rustic Server Hold Mah Dick");

        match (&r.method, &r.request_uri) {
            (&Get, &AbsolutePath(path)) => {

            },
            (&Post, &AbsolutePath(path)) => {

            },
            (&Put, &AbsolutePath(path)) => {

            },
            (&Delete, &AbsolutePath(path)) => {

            }
        }
    }
}