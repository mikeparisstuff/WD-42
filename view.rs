#![feature(globs)]

extern crate http;

use http::server::{Request, ResponseWriter};

use std::io::*;
// The View struct holds its relevant routes

#[deriving(Clone)]
pub struct View {
    get: fn(request: &Request, response: &mut ResponseWriter),
    post: fn(request: &Request, response: &mut ResponseWriter),
    put: fn(request: &Request, response: &mut ResponseWriter),
    delete: fn(request: &Request, response: &mut ResponseWriter)
}

fn main() {
    println!("Compiled view.rs")
}

