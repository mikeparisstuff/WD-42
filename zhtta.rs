//
// zhtta.rs
//
// Starting code for PS3
// Running on Rust 0.9
//
// Note that this code has serious security risks!  You should not run it 
// on any system with access to sensitive files.
// 
// University of Virginia - cs4414 Spring 2014
// Weilin Xu and David Evans
// Version 0.5

// To see debug! outputs set the RUST_LOG environment variable, e.g.: export RUST_LOG="zhtta=debug"

#[feature(globs)];
extern mod extra;

use std::io::*;
use std::io::buffered::BufferedReader;
use std::io::File;
use std::io::net::ip::{SocketAddr};
use std::{os, str, libc, from_str};
use std::path::Path;
use std::hashmap::HashMap;
use extra::sync::Semaphore;

use extra::getopts;
use extra::arc::MutexArc;
use extra::arc::RWArc;

pub mod gash;

static SERVER_NAME : &'static str = "Zhtta Version 0.5";

static IP : &'static str = "127.0.0.1";
static PORT : uint = 4414;
static WWW_DIR : &'static str = "./www";

static HTTP_OK : &'static str = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n";
static HTTP_BAD : &'static str = "HTTP/1.1 404 Not Found\r\n\r\n";

static COUNTER_STYLE : &'static str = "<doctype !html><html><head><title>Hello, Rust!</title>
             <style>body { background-color: #884414; color: #FFEEAA}
                    h1 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red }
                    h2 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green }
             </style></head>
             <body>";

// static mut visitor_count : uint = 0;

static mut counter : uint = 0;

struct HTTP_Request {
    // Use peer_name as the key to access TcpStream in hashmap. 

    // (Due to a bug in extra::arc in Rust 0.9, it is very inconvenient to use TcpStream without the "Freeze" bound.
    //  See issue: https://github.com/mozilla/rust/issues/12139)
    peer_name: ~str,
    path: ~Path,
}

struct WebServer {
    ip: ~str,
    port: uint,
    www_dir_path: ~Path,
    
    request_queue_arc: MutexArc<~[HTTP_Request]>,
    stream_map_arc: MutexArc<HashMap<~str, Option<std::io::net::tcp::TcpStream>>>,
    cache_arc: RWArc<HashMap<~str, ~[u8]>>,

    visitor_count_arc: MutexArc<int>,
    
    notify_port: Port<()>,
    shared_notify_chan: SharedChan<()>,

    task_semaphore: Semaphore,
}

impl WebServer {
    fn new(ip: &str, port: uint, www_dir: &str) -> WebServer {
        let (notify_port, shared_notify_chan) = SharedChan::new();
        let www_dir_path = ~Path::new(www_dir);
        os::change_dir(www_dir_path.clone());

        WebServer {
            ip: ip.to_owned(),
            port: port,
            www_dir_path: www_dir_path,
                        
            request_queue_arc: MutexArc::new(~[]),
            stream_map_arc: MutexArc::new(HashMap::new()),

            visitor_count_arc: MutexArc::new(0),
            cache_arc: RWArc::new(HashMap::new()),
            
            notify_port: notify_port,
            shared_notify_chan: shared_notify_chan,

            task_semaphore : Semaphore::new(4),
        }
    }
    
    fn run(&mut self) {
        self.listen();
        self.dequeue_static_file_request();
    }
    
    fn listen(&mut self) {
        let addr = from_str::<SocketAddr>(format!("{:s}:{:u}", self.ip, self.port)).expect("Address error.");
        let www_dir_path_str = self.www_dir_path.as_str().expect("invalid www path?").to_owned();

        let request_queue_arc = self.request_queue_arc.clone();
        let shared_notify_chan = self.shared_notify_chan.clone();
        let stream_map_arc = self.stream_map_arc.clone();

        let visitor_count_arc = self.visitor_count_arc.clone();

        // First task listens for connections from the port
        spawn(proc() {
            let mut acceptor = net::tcp::TcpListener::bind(addr).listen();
            println!("{:s} listening on {:s} (serving from: {:s}).", 
                     SERVER_NAME, addr.to_str(), www_dir_path_str);

            for stream in acceptor.incoming() {
                let (queue_port, queue_chan) = Chan::new();
                queue_chan.send(request_queue_arc.clone());
                
                let notify_chan = shared_notify_chan.clone();
                let stream_map_arc = stream_map_arc.clone();

                // Counter Chan and Port
                let (counter_port, counter_chan) = Chan::new();
                counter_chan.send(visitor_count_arc.clone());

                // Spawn a task to handle the connection.
                // Next N tasks handle each of the port's N connections
                // Thus, there are "N+1" tasks running for N connections
                spawn(proc() {
                    // Safe Visitor Count
                    // Mutex Arc gives access to underlying visitor count
                    let visitor_count_arc = counter_port.recv();
                    let mut current_visitor_count : int = 0;
                    visitor_count_arc.access(|val| {
                        *val += 1;
                        current_visitor_count = *val;
                        // println!("Visitor Number: {:d}", *val);
                    });

                    // unsafe { visitor_count += 1; } // TODO: Fix unsafe counter

                    let request_queue_arc = queue_port.recv();
                  
                    let mut stream = stream;
                    
                    let peer_name = WebServer::get_peer_name(&mut stream);

                    let mut buf = [0, ..500];
                    stream.read(buf);
                    let request_str = str::from_utf8(buf);
                    debug!("Request:\n{:s}", request_str);
                    
                    let req_group : ~[&str]= request_str.splitn(' ', 3).collect();
                    if req_group.len() > 2 {
                        let path_str = "." + req_group[1].to_owned();
                        
                        let mut path_obj = ~os::getcwd();
                        path_obj.push(path_str.clone());
                        
                        let ext_str = match path_obj.extension_str() {
                            Some(e) => e,
                            None => "",
                        };
                        
                        debug!("Requested path: [{:s}]", path_obj.as_str().expect("error"));
                        debug!("Requested path: [{:s}]", path_str);
                             
                        if path_str == ~"./" {
                            // debug!("===== Counter Page request =====");
                            println!("===== Counter Page request =====");
                            WebServer::respond_with_counter_page(stream, current_visitor_count);
                            debug!("=====Terminated connection from [{:s}].=====", peer_name);
                        } else if !path_obj.exists() || path_obj.is_dir() {
                            // debug!("===== Error page request =====");
                            println!("===== Error page request =====");
                            WebServer::respond_with_error_page(stream, path_obj);
                            debug!("=====Terminated connection from [{:s}].=====", peer_name);
                        } else if ext_str == "shtml" { // Dynamic web pages.
                            // debug!("===== Dynamic Page request =====");
                            println!("===== Dynamic Page request =====");
                            WebServer::respond_with_dynamic_page(stream, path_obj);
                            debug!("=====Terminated connection from [{:s}].=====", peer_name);
                        } else { 
                            // debug!("===== Static Page request =====");
                            println!("===== Static Page request =====");
                            WebServer::enqueue_static_file_request(stream, path_obj, stream_map_arc, request_queue_arc, notify_chan);
                        }
                    }
                });
            }
        });
    }

    fn respond_with_error_page(stream: Option<std::io::net::tcp::TcpStream>, path: &Path) {
        let mut stream = stream;
        let msg: ~str = format!("Cannot open: {:s}", path.as_str().expect("invalid path").to_owned());

        stream.write(HTTP_BAD.as_bytes());
        stream.write(msg.as_bytes());
    }

    // TODO: Safe visitor counter.
    fn respond_with_counter_page(stream: Option<std::io::net::tcp::TcpStream>, count:int) {
        let mut stream = stream;
        let response: ~str = 
            format!("{:s}{:s}<h1>Greetings, Krusty!</h1>
                     <h2>Visitor count: {:d}</h2></body></html>\r\n", 
                    HTTP_OK, COUNTER_STYLE,
                    count ); //unsafe { visitor_count }
        debug!("Responding to counter request");
        stream.write(response.as_bytes());
    }
    
    // TODO: Streaming file.
    // TODO: Application-layer file caching.
    fn respond_with_static_file(stream: Option<std::io::net::tcp::TcpStream>, path: &Path, cache_arc: RWArc<HashMap<~str, ~[u8]>>) {
        // println!("Responding with static file...");


        // println!("Responding with static file...");
        // let mut stream = stream;
        // let bytes_to_read = path.stat().size.to_uint().unwrap();
        // static buffer_size: uint = 4096*10;

        // let mut file_reader = BufferedReader::new(File::open(path));

        // stream.write(HTTP_OK.as_bytes());

        // let iterations = bytes_to_read / buffer_size;
        // let num_trailing_bytes = bytes_to_read % buffer_size;

        // for i in range(0,iterations){
        //     stream.write(file_reader.read_bytes(buffer_size));
        // }
        // if num_trailing_bytes != 0 {
        //     stream.write(file_reader.read_bytes(num_trailing_bytes));
        // }
        // println("Served");






        let cache_arc = cache_arc.clone();
        let key: ~str = path.as_str().clone().unwrap().to_owned();
        let mut cache_miss: bool = false;
        let mut file_bytes: ~[u8] = ~[];
        let mut stream = stream;

        cache_arc.read(|cache| {
            if cache.contains_key(&key) {
                println!("Cache hit!");
                let content = cache.get(&key).clone();
                stream.write(HTTP_OK.as_bytes());
                stream.write(content.clone().to_owned());
                println!("File served");
            } else {
                println!("Cache miss! Responding with static file");

                let bytes_to_read = path.stat().size.to_uint().unwrap();
                static buffer_size: uint = 4096;

                let mut file_reader = BufferedReader::new(File::open(path));

                stream.write(HTTP_OK.as_bytes());

                let iterations = bytes_to_read / buffer_size;
                let num_trailing_bytes = bytes_to_read % buffer_size;

                let mut bytes_read: ~[u8] = ~[];
                for i in range(0,iterations){
                    bytes_read = file_reader.read_bytes(buffer_size); 
                    stream.write(bytes_read);
                    file_bytes = std::vec::append(file_bytes, bytes_read)
                }
                if num_trailing_bytes != 0 {
                    bytes_read = file_reader.read_bytes(num_trailing_bytes); 
                    stream.write(bytes_read);
                    file_bytes = std::vec::append(file_bytes, bytes_read)
                }
                println!("File served");

                // Cache it!
                cache_miss = true;
            }
        });

        if cache_miss {
            // Cache the data!
            println!("Caching file!");

            let mut fits_in_cache: bool = false;
            // Size of cache = 1024 bytes
            cache_arc.read(|cache| {
                if (path.stat().size.to_uint().unwrap() + cache.len()) <= 5120 {
                    fits_in_cache = true;
                    println!("Fits in cache {}", fits_in_cache);
                }
            });

            if fits_in_cache {
                cache_arc.write(|cache| {
                    cache.insert(key.clone(), file_bytes.clone());
                });
            }
        }
    }
    
    // TODO: Server-side gashing.
    fn respond_with_dynamic_page(stream: Option<std::io::net::tcp::TcpStream>, path: &Path) {
        // Read in a file and extract command lines
        let mut file = BufferedReader::new(File::open(path));

        let mut file_contents : ~str = ~"";
        for line in file.lines() {
            // println(line);
            match line.find_str("<!--") {
                None => { 
                    /*println!("No commands in line");*/ 
                    file_contents = file_contents + line;
                }
                Some(i) => { 
                    // We found a left index for the command. Find the right
                    match line.find_str("-->") {
                        None => { println!("Could not find end of comment."); }
                        Some(end) => { 
                            let cmd_str : &str = line.slice(i, end+3);
                            // println!("Found command: {:s}", cmd_str);

                            // Found a command. Now run the command in gash and replace the
                            // contents of the line with the result
                            let mut cmd : &str = "";
                            match cmd_str.find('"') {
                                None => { /* Do Nothing */ }
                                Some(beg) => { 
                                    match cmd_str.rfind('"') {
                                        None => { }
                                        Some(end_cmd) => {
                                            cmd = cmd_str.slice(beg+1, end_cmd);
                                            // Recreate the full line of the file with the replaced value
                                            let first : &str = line.slice_to(i);
                                            let last : &str = line.slice_from(end+3);
                                            let ret_str = gash::run_cmdline(cmd);
                                            let full_line = first + ret_str + last;
                                            file_contents = file_contents + full_line;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Serve the file_contents
        let mut stream = stream;
        stream.write(HTTP_OK.as_bytes());
        stream.write(file_contents.as_bytes());

        // WebServer::respond_with_static_file(stream, path);
    }
    
    // TODO: Smarter Scheduling.
    fn enqueue_static_file_request(stream: Option<std::io::net::tcp::TcpStream>, path_obj: &Path, stream_map_arc: MutexArc<HashMap<~str, Option<std::io::net::tcp::TcpStream>>>, req_queue_arc: MutexArc<~[HTTP_Request]>, notify_chan: SharedChan<()>) {
        // Save stream in hashmap for later response.
        let mut stream = stream;
        let peer_name = WebServer::get_peer_name(&mut stream);
        let (stream_port, stream_chan) = Chan::new();
        stream_chan.send(stream);
        unsafe {
            // Use an unsafe method, because TcpStream in Rust 0.9 doesn't have "Freeze" bound.
            stream_map_arc.unsafe_access(|local_stream_map| {
                let stream = stream_port.recv();
                local_stream_map.swap(peer_name.clone(), stream);
            });
        }
        
        // Enqueue the HTTP request.
        let req = HTTP_Request { peer_name: peer_name.clone(), path: ~path_obj.clone() };
        let (req_port, req_chan) = Chan::new();
        req_chan.send(req);

        let mut ip_addr: ~str = ~"";
        let mut ip_addr_beginning: ~str = ~"";

        match peer_name.find(':') {
            None => {}
            Some(ip_end) => {
                let ip_addr : ~str;
                ip_addr = peer_name.slice(0, ip_end).to_owned();
                ip_addr_beginning = ip_addr.slice(0, 8).to_owned();
                // println!("sliced: {}", ip_addr.slice(0, 8))
            }
        }
        println!("sliced: {}", ip_addr_beginning)

        debug!("Waiting for queue mutex lock.");
        req_queue_arc.access(|local_req_queue| {
            debug!("Got queue mutex lock.");
            let req: HTTP_Request = req_port.recv();
            // local_req_queue.push(req);
            debug!("A new request enqueued, now the length of queue is {:u}.", local_req_queue.len());

            // ip_addr_beginning = ~"128.143.";

            match ip_addr_beginning {
                ~"128.143." => {
                    // println("First");
                    local_req_queue.insert(0, req)
                }
                ~"137.54." => {
                    // println("Second");
                    local_req_queue.insert(0, req)
                }
                _ => {
                    // println("Third");
                    local_req_queue.push(req)
                }
            }
        });
        
        notify_chan.send(()); // Send incoming notification to responder task.
    
    
    }


    
    // TODO: Smarter Scheduling.
    fn dequeue_static_file_request(&mut self) {
        let req_queue_get = self.request_queue_arc.clone();
        let stream_map_get = self.stream_map_arc.clone();
        let cache_arc = self.cache_arc.clone();
        
        // Port<> cannot be sent to another task. So we have to make this task as the main task that can access self.notify_port.
        
        let (request_port, request_chan) = Chan::new();
        let (semaphore_port, semaphore_chan) = Chan::new();
        let (cache_port, cache_chan) = Chan::new();

        loop {
            self.notify_port.recv();    // waiting for new request enqueued.
            
            req_queue_get.access( |req_queue| {

                //Instead of FIFO queue, take smallest file from front X requests
                let limit_constant = 5;
                let limit = std::num::min( limit_constant, req_queue.len() );

                let mut index = 0;
                let mut min_size = req_queue[0].path.stat().size;

                for i in range(1,limit){
                    if (req_queue[i].path.stat().size < min_size){
                        index = i;
                        min_size = req_queue[i].path.stat().size;
                    }
                }
                // println!("Serving request at index: {}",index);
                let req = req_queue.remove(index);
                request_chan.send(req);

                //OLD CODE
                // match req_queue.shift_opt() { // FIFO queue.
                //     None => { /* do nothing */ }
                //     Some(req) => {
                //         request_chan.send(req);
                //         println!("A new request dequeued, now the length of queue is {:u}.", req_queue.len());
                //     }
                // }
            });
            
            let request = request_port.recv();
            
            // Get stream from hashmap.
            // Use unsafe method, because TcpStream in Rust 0.9 doesn't have "Freeze" bound.
            let (stream_port, stream_chan) = Chan::new();
            unsafe {
                stream_map_get.unsafe_access(|local_stream_map| {
                    let stream = local_stream_map.pop(&request.peer_name).expect("no option tcpstream");
                    stream_chan.send(stream);
                });
            }
            
            // TODO: Spawning more tasks to respond the dequeued requests concurrently. You may need a semophore to control the concurrency.
            semaphore_chan.send(request);
            cache_chan.send(cache_arc.clone());

            self.task_semaphore.access(|| {
                let request = semaphore_port.recv();
                let stream = stream_port.recv();
                let cache_arc_copy = cache_port.recv();

                let (cache_p, cache_c) = Chan::new();
                cache_c.send(cache_arc_copy);

                spawn(proc() {
                    let cache = cache_p.recv();
                    WebServer::respond_with_static_file(stream, request.path, cache);
                });
            });

            // let request = semaphore_port.recv();

            // Without helper threads
            // let stream = stream_port.recv();
            // WebServer::respond_with_static_file(stream, request.path);

            // Close stream automatically.
            // debug!("=====Terminated connection from [{:s}].=====", request.peer_name);
        }
    }
    
    fn get_peer_name(stream: &mut Option<std::io::net::tcp::TcpStream>) -> ~str {
        match *stream {
            Some(ref mut s) => {
                         match s.peer_name() {
                            Some(pn) => {pn.to_str()},
                            None => (~"")
                         }
                       },
            None => (~"")
        }
    }
}

fn get_args() -> (~str, uint, ~str) {
    fn print_usage(program: &str) {
        println!("Usage: {:s} [options]", program);
        println!("--ip     \tIP address, \"{:s}\" by default.", IP);
        println!("--port   \tport number, \"{:u}\" by default.", PORT);
        println!("--www    \tworking directory, \"{:s}\" by default", WWW_DIR);
        println("-h --help \tUsage");
    }
    
    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();
    let program = args[0].clone();
    
    let opts = ~[
        getopts::optopt("ip"),
        getopts::optopt("port"),
        getopts::optopt("www"),
        getopts::optflag("h"),
        getopts::optflag("help")
    ];

    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_err_msg()) }
    };

    if matches.opt_present("h") || matches.opt_present("help") {
        print_usage(program);
        unsafe { libc::exit(1); }
    }
    
    let ip_str = if matches.opt_present("ip") {
                    matches.opt_str("ip").expect("invalid ip address?").to_owned()
                 } else {
                    IP.to_owned()
                 };
    
    let port:uint = if matches.opt_present("port") {
                        from_str::from_str(matches.opt_str("port").expect("invalid port number?")).expect("not uint?")
                    } else {
                        PORT
                    };
    
    let www_dir_str = if matches.opt_present("www") {
                        matches.opt_str("www").expect("invalid www argument?") 
                      } else { WWW_DIR.to_owned() };
    
    (ip_str, port, www_dir_str)
}

fn main() {
    let (ip_str, port, www_dir_str) = get_args();
    let mut zhtta = WebServer::new(ip_str, port, www_dir_str);
    zhtta.run();
}
