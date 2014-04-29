#![feature(globs)]

extern crate collections;
extern crate serialize;
extern crate http;

use http::server::{Request, ResponseWriter};
use http::status::{BadRequest, MethodNotAllowed, Ok};
use std::io::println;
use std::io::BufWriter;
use std::io;
use std::Vec;
use collections::HashMap;
use serialize::json;
use serialize::json::ToJson;
use collections::TreeMap;
mod application;

struct PayloadExample {
    first_name: ~str,
    last_name: ~str,
}

impl ToJson for PayloadExample {
    fn to_json( &self ) -> json::Json {
        let mut d = ~TreeMap::new();
        d.insert("first_name".to_owned(), self.first_name.to_json());
        d.insert("last_name".to_owned(), self.last_name.to_json());
        json::Object(d)
    }
}

fn indexGet(req: &Request, res: &mut ResponseWriter) {
    println!("Hello get!");
    res.status = Ok;
    res.sendFile(~"index.html");
}
fn indexPost(req: &Request, res: &mut ResponseWriter) {
    println!("Received Post with body: \n{}", req.body);
}

fn encodeGet(req: &Request, res: &mut ResponseWriter) {
    let payload : PayloadExample = PayloadExample { first_name: ~"Johnny", last_name: ~"Bravo" };
    let pljson : ~str = payload.to_json().to_str();
    res.write(pljson.as_bytes());
}

fn noFileGet(req: &Request, res: &mut ResponseWriter) {
    res.status = Ok;
    res.sendFile(~"nobedere.html");
}

fn authenticate(req: &mut Request) {
    let allowedUsers : [~str, ..4] = [~"mlp5ab", ~"ag7bf", ~"bp5xj", ~"nal3vm"];

    let mut headers = req.headers.clone();
    let mut value: ~str = match headers.get_header(~"Authorization") {
        Some(value) => {value},
        None => {~"THIS IS NOT A USER LOL"}
    };
    println!("Auth value: {}", value);

    //TODO: change req.is_authenticated..
    if allowedUsers.contains(&value) {
        req.is_authenticated = true;
        println!("User Authenticated");
    }
    else {
        req.is_authenticated = false;
        println!("Request not authenticated");
    }
}

fn checkAuth(req: &mut Request) {
    println!("This request is authenticated: {}", req.is_authenticated);
}


fn main() {

    let mut app: application::App = application::App::new();

    // TODO: Use actual request and response objects

    app.set_public_dir(~"./public");

    app.get(~"/", indexGet);
    app.post(~"/", indexPost);
    app.get(~"/encode", encodeGet);
    app.get(~"/wrong", noFileGet);
    app.apply(authenticate);
    app.apply(checkAuth);

    println!("{}", app)
    app.listen(8000);
}
