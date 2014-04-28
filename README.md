WD-42
======

Welcome to the WD-42 repo! WD-42 is a lightweight web framework that helps take the Rust off. 


###Quick Start
Once you have downloaded and installed **WD-42**, you simply need 5 lines of code to make your first endpoint, apply middleware, and launch your website!
```rust
let mut app: application::App = application::App::new();
app.set_public_dir(~"./public"); // Set your public folder to serve files from
app.get(~"/", indexGet);         // Pass a path and method to declare an endpoint
app.apply(authenticate);         // Apply middleware by passing the necessary function
app.listen(8000);                // Default port is 8000
```
Your public folder must be in the same directory as your main file. To launch your website, `cd` to where your main file is, `rustc` that sucker, and then `./[your-file-name]`!

A simple function to handle an endpoint looks like so:
```rust
fn indexGet(req: &Request, res: &mut ResponseWriter) {
    res.status = Ok;
    res.sendFile(~"index.html");
}
```

WD-42 supports `GET`, `POST`, `PUT`, and `DELETE`. 
```rust
app.get(path, fn);
app.post(path, fn);
app.put(path, fn);
app.delete(path, fn);
```

