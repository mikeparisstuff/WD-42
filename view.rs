#![feature(globs)]
use std::io::*;

// The View struct holds its relevant routes

pub struct View {
    get: fn(request: ~str, response: ~str) -> ~str,
    post: fn(request: ~str, response: ~str) -> ~str,
    put: fn(request: ~str, response: ~str) -> ~str,
    delete: fn(request: ~str, response: ~str) -> ~str
}

fn main() {
    println!("Compiled view.rs")
}

