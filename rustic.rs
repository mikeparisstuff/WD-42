#![feature(globs)]

extern crate collections;
extern crate http;

use http::server::{Request, ResponseWriter};
use http::status::{BadRequest, MethodNotAllowed, Ok};
use std::io::println;
use std::Vec;
use collections::HashMap;
mod application;

fn main() {

    let mut app: application::App = application::App::new();

    // TODO: Use actual request and response objects
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
        println!("Length of headers: {}", req.headers.extensions.len());
        match req.headers.extensions.find(&~"Authorization") {
            Some(user) => {
                if allowedUsers.contains(user) {
                    println!("Successfully authenticated user: {}", user);
                } else {
                    println!("Could not authenticate user: {}", user);
                }
            },
            None => {
                println("Could not find Authorization in headers");
            }
        }
    }

    fn checkAuth(req: &Request) {
        println!("This request is authenticated: {}", req.is_authenticated);
    }

    app.set_public_dir(~"./public");

    // fn indexPut(req: &Request, res: &mut ResponseWriter) {
    //     println!("Hello put!");
    // }
    // fn indexDelete(req: &Request, res: &mut ResponseWriter) {
    //     println!("Hello delete!");
    // }

    app.get(~"/", indexGet);
    app.post(~"/", indexPost);
    app.get(~"/wrong", noFileGet);
    app.apply(authenticate);
    app.apply(checkAuth);

    println!("{}", app)
    app.listen(8000);
}
