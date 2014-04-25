#![feature(globs)]

extern crate collections;
extern crate http;

use http::server::{Request, ResponseWriter};
use std::io::println;
use collections::HashMap;
mod application;

fn main() {
    let mut app: application::App = application::App::new();
    let mut urls = ~HashMap::new();

    // TODO: Use actual request and response objects
    fn indexGet(req: &Request, res: &mut ResponseWriter) {
        println!("Hello get!");
    }
    fn indexPost(req: &Request, res: &mut ResponseWriter) {
        println!("Hello post!");
    }
    fn indexPut(req: &Request, res: &mut ResponseWriter) {
        println!("Hello put!");
    }
    fn indexDelete(req: &Request, res: &mut ResponseWriter) {
        println!("Hello delete!");
    }

    let indexView: application::view::View = application::view::View {
        get: indexGet,
        post: indexPost,
        put: indexPut,
        delete: indexDelete
    };
    urls.insert(~"/", indexView);
    app.setRoutes(urls);
    println!("{}", app)
    app.listen(8000);
}
