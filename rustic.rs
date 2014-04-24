#![feature(globs)]

extern crate collections;
extern crate http;
use std::io::println;
use collections::HashMap;
mod request;
mod response;
mod application;

fn main() {
    let mut myServer: application::App = application::App::new();
    let mut urls = ~HashMap::new();

    // TODO: Use actual request and response objects
    fn indexGet(req: ~str, res: ~str) -> ~str {
        ~"Hello get!"
    }
    fn indexPost(req: ~str, res: ~str) -> ~str {
        ~"Hello post!"
    }
    fn indexPut(req: ~str, res: ~str) -> ~str {
        ~"Hello put!"
    }
    fn indexDelete(req: ~str, res: ~str) -> ~str {
        ~"Hello delete!"
    }

    let indexView: application::view::View = application::view::View { 
        get: indexGet,
        post: indexPost,
        put: indexPut,
        delete: indexDelete
    };
    urls.insert(~"/", indexView);
    myServer.setRoutes(urls);

    println!("{}", myServer)
}
