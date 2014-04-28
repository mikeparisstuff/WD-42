#![feature(globs)]

extern crate collections;
extern crate http;

use http::server::{Request, ResponseWriter};
use http::status::{BadRequest, MethodNotAllowed, Ok};
use std::io::println;
use std::io::BufWriter;
use std::io;
use std::Vec;
use collections::HashMap;
mod application;

fn indexGet(req: &Request, res: &mut ResponseWriter) {
    println!("Hello get!");
    res.status = Ok;
    res.sendFile(~"index.html");
    // res.write(bytes!("Hello World!"));
}
fn indexPost(req: &Request, res: &mut ResponseWriter) {
    println!("Hello post!");
}

fn noFileGet(req: &Request, res: &mut ResponseWriter) {
    res.status = Ok;
    res.sendFile(~"nobedere.html");
}

fn authenticate(req: &Request) {
    let allowedUsers : [~str, ..4] = [~"mlp5ab", ~"ag7bf", ~"bp5xj", ~"nal3vm"];

    let mut headers = req.headers.clone();
    let mut value: ~str = match headers.get_header(~"Authorization") {
        Some(value) => {value},
        None => {~"THIS IS NOT A USER LOL"}
    };
    println!("Auth value: {}", value);
    //TODO: change req.is_authenticated..
    if allowedUsers.contains(&value) {
        println!("User Authenticated");
    }
    else {
        println!("Request not authenticated");
    }

}

fn checkAuth(req: &Request) {
    println!("This request is authenticated: {}", req.is_authenticated);
}


fn main() {

    let mut app: application::App = application::App::new();

    // TODO: Use actual request and response objects

    app.set_public_dir(~"./public");

    app.get(~"/", indexGet);
    app.post(~"/", indexPost);
    app.get(~"/wrong", noFileGet);
    app.apply(authenticate);
    app.apply(checkAuth);

    println!("{}", app)
    app.listen(8000);
}
