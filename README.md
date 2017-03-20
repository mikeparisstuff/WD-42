# WD-42 - The lightweight Web Framework for Rust
------

NOTE: This was desined for Rust .10 and will most likely need some tinkering to be compatible with more recent rust versions

Welcome to the WD-42 repo! WD-42 is a lightweight web framework that helps take the Rust off.

## WD-42 Breakdown
WD-42 is broken into two main parts.

1) The WD-42 Application found in application.rs

   * Most user interaction with this framework is done by interacting with an instance of this application.

2) Rust-Http

   * A library created by Chris Morgan that we have extended for our own purposes.

## Quick Start

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
app.get(path, fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>));
app.post(path, fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>));
app.put(path, fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>));
app.delete(path, fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>));
```

WD-42 supports arbitrary middleware as long as it has the following signature.
```rust
fn(&mut http::server::request::Request)
```

## JSON Serialization
Rust comes packaged with a json serializer that we can use to serialize our structs to pass over the network. Doing this is simple and requires you to simply implement a single **ToJson** Trait for your struct.

For Example:
```rust
// Define a struct
struct PayloadExample {
    first_name: ~str,
    last_name: ~str,
}

// Define the ToJson Trait on this struct
impl ToJson for PayloadExample {
    fn to_json( &self ) -> json::Json {
        let mut d = ~TreeMap::new();
        d.insert("first_name".to_owned(), self.first_name.to_json());
        d.insert("last_name".to_owned(), self.last_name.to_json());
        json::Object(d)
    }
}

// And then serializing the struct is as simple as
fn encodeGet(req: &Request, res: &mut ResponseWriter) {
    let payload : PayloadExample = PayloadExample { first_name: ~"Johnny", last_name: ~"Bravo" };
    let pljson : ~str = payload.to_json().to_str();
    res.write(pljson.as_bytes());
}

// You can then register this fn as a request handler using app.get() to see the result
app.get(~"/encode", encodeGet);
```

### Contribution
If you would like to add features and/or make changes to this repo please feel free to fork it and submit pull requests.  We  are aware that the rust-http package is going to be phased out for a better solution in the future and thus this project will likely undergo large, non-backwards compatible changes in the future.

Good Hacking!
