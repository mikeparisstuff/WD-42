#WD-42
------

Welcome to the WD-42 repo! WD-42 is a lightweight web framework that helps take the Rust off.

##WD-42 Breakdown
WD-42 is broken into two main parts.

1) The WD-42 Application found in application.rs: Most user interaction with this framework is done by interacting with and instance of this application.

2) Rust-Http: A library created by Chris Morgan that we have extended for our own purposes.

##Quick Start

To download WD-42 simply clone or download the zip file for this repo and run **make run** within the root project directory.  This should compile and start the example site within server.rs. By default the makefile looks for the file 'server.rs' to hold the user level application, but you can change this behavior in the Makefile.

Once you have downloaded and installed **WD-42**, you need minimal code to make your first endpoint, apply custom middleware, and launch your website!
```rust
fn indexGet(req: &Request, res: &mut ResponseWriter) {
    println!("Hello get!");
    res.status = Ok;
    res.sendFile(~"index.html");
}

fn authenticate(req: &mut Request) {
    let mut headers = req.headers.clone();
    let mut value: ~str = match headers.get_header(~"Authorization") {
        Some(value) => {value},
        None => {~"Could not find user"}
    };
    ... some database query to look up users
    if authorizedUsers.contains(&value) {
        req.is_authenticated = true;
        println!("User Authenticated");
    }
}

fn main() {
    let mut app: application::App = application::App::new();
    app.set_public_dir(~"./public"); // Set your public folder to serve files from
    app.get(~"/", indexGet);         // Pass a path and method to declare an endpoint
    app.apply(authenticate);         // Apply middleware by passing the necessary function
    app.listen(8000);                // Default port is 8000
}
```
Your public folder should be the place where you load all your html, css, images, and js files that will be sent to the browser. All calls to sendFile() are relative to this public folder.

WD-42 supports `GET`, `POST`, `PUT`, and `DELETE`.
```rust
app.get(path, fn);
app.post(path, fn);
app.put(path, fn);
app.delete(path, fn);
```
