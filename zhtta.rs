//
// zhtta.rs
//
// Reference code for PS3
// Running on Rust 0.9
//
// Note that this code has serious security risks!  You should not run it
// on any system with access to sensitive files.
//
// University of Virginia - cs4414 Spring 2014
// Weilin Xu and David Evans
// Version 0.5

// To see debug! outputs set the RUST_LOG environment variable, e.g.: export RUST_LOG="zhtta=debug"

#![feature(globs)]
#![feature(phase)]
#[phase(syntax, link)] extern crate log;

// extern crate extra;

extern crate std;
extern crate collections;
extern crate sync;
extern crate getopts;

use std::io::*;
use std::io::net::ip::{SocketAddr};
use std::{os, str, libc, from_str};
use std::comm::{Sender, Receiver, channel};

use collections::hashmap::HashMap;
use collections::priority_queue::PriorityQueue;

use sync::raw::Semaphore;
use sync::{Arc, Mutex, RWLock};



// mod gash;

static IP: &'static str = "127.0.0.1";
static PORT:        uint = 4414;
static WWW_DIR: &'static str = "./www";
static MAX_CONCURRENCY: uint = 15;
static CHUNK_SIZE: uint = 50000;

struct HTTP_Request {
     // Use peer_name as the key to TcpStream.
     // Due to a bug in extra::arc in Rust 0.9, it is very inconvenient to use TcpStream without the "Freeze" bound.
     // Issue: https://github.com/mozilla/rust/issues/12139
    peer_name: ~str,
    path: ~std::path::PosixPath,
    file_size: uint,
    priority: uint,
}

impl std::cmp::Ord for HTTP_Request {

    fn lt(&self, other: &HTTP_Request) -> bool {
        if self.priority > other.priority { true } else { false }
    }
}

impl std::cmp::Eq for HTTP_Request {

    fn eq(&self, other: &HTTP_Request) -> bool {
        self.priority == other.priority
    }
}

struct CacheItem {
    file_path: ~str,
    data: ~[u8],
    file_size: uint,
    status: int, // 0: available, 1: updating, -1: not exist.
    //last_modified_time: u64,
}

struct WebServer {
    ip: ~str,
    port: uint,
    www_dir_path: ~Path,
    max_concurrency: uint,
    file_chunk_size: uint,

    concurrency_sem: Semaphore,
    visitor_count_arc: Arc<RWLock<uint>>,
    request_queue_arc: Arc<Mutex<PriorityQueue<HTTP_Request>>>,
    stream_map_arc: Arc<Mutex<HashMap<~str, Option<std::io::net::tcp::TcpStream>>>>,

    notify_port: Receiver<()>,
    shared_notify_chan: Sender<()>,

    // `std::hashmap::HashMap<~str,extra::arc::Arc<CacheItem>>` does not fulfill `Freeze`
    // So I have to use the unsafe method in Mutex instead.
    cache_arc: Arc<Mutex<HashMap<~str, Arc<CacheItem>>>>,
}

impl WebServer {
    fn new(ip: &str, port: uint, www_dir: &str, max_concurrency: uint, file_chunk_size: uint) -> WebServer {
        // TODO: chroot jail
        let www_dir_path = ~Path::new(www_dir);
        os::change_dir(www_dir_path.clone());
        let (shared_notify_chan, notify_port) = channel();
        WebServer {
            ip: ip.to_owned(),
            port: port,
            www_dir_path: www_dir_path,
            max_concurrency: max_concurrency,
            file_chunk_size: file_chunk_size,

            visitor_count_arc: Arc::new(RWLock::new(0 as uint)),
            concurrency_sem: Semaphore::new(max_concurrency as int),

            request_queue_arc: Arc::new(Mutex::new(PriorityQueue::new())),
            stream_map_arc: Arc::new(Mutex::new(HashMap::new())),

            notify_port: notify_port,
            shared_notify_chan: shared_notify_chan,

            cache_arc: Arc::new(Mutex::new(HashMap::new())),
        }
    }


    fn run(&mut self) {
        self.listen();
        self.dequeue_static_file_request();
    }

    fn listen(&mut self) {
        // Create socket.
        let addr = from_str::<SocketAddr>(format!("{:s}:{:u}", self.ip, self.port)).expect("Address error.");
        let www_dir_path_str = self.www_dir_path.as_str().expect("invalid www path?").to_owned();

        let request_queue_arc = self.request_queue_arc.clone();
        let shared_notify_chan = self.shared_notify_chan.clone();
        let stream_map_arc = self.stream_map_arc.clone();
        let visitor_count_arc = self.visitor_count_arc.clone();
        let cache_arc = self.cache_arc.clone();
        let file_chunk_size = self.file_chunk_size;

        spawn (proc(){
            let mut acceptor = net::tcp::TcpListener::bind(addr).listen();
            println!("Listening on [{:s}] ...", addr.to_str());
            println!("Working directory in [{:s}].", www_dir_path_str);

            for stream in acceptor.incoming() {
                let (queue_chan, queue_port) = channel();
                queue_chan.send(request_queue_arc.clone());

                let notify_chan = shared_notify_chan.clone();
                let stream_map_arc = stream_map_arc.clone();
                let cache_arc = cache_arc.clone();
                //let file_chunk_size = file_chunk_size;
                let visitor_count_arc = visitor_count_arc.clone();
                // Spawn a task to handle the connection
                std::task::spawn (proc(){
                    // visitor_count_arc.write(|count| {
                    //     *count += 1;
                    // });
                    let mut count = visitor_count_arc.write();
                    *count += 1;
                    let count = count.downgrade();
                    let req_queue_arc = queue_port.recv();

                    let mut stream = stream.ok();

                    let peer_name = WebServer::get_peer_name(&mut stream);
                    debug!("=====Received connection from: [{:s}]=====", peer_name);

                    let mut buf = [0, ..500];
                    stream.unwrap().read(buf);
                    let request_str = str::from_utf8(buf).unwrap();
                    debug!("Request :\n{:s}", request_str);

                    let req_group : ~[&str]= request_str.splitn(' ', 3).collect();
                    if req_group.len() > 2 {
                        let path_str = "." + req_group[1].to_owned();

                        let mut path_obj = ~os::getcwd();
                        path_obj.push(path_str.clone());

                        let ext_str = match path_obj.extension_str() {
                            Some(e) => e,
                            None => "",
                        };

                        if !path_obj.exists() || path_obj.is_dir() {
                            WebServer::respond_with_default_page(stream, visitor_count_arc);
                            debug!("=====Terminated connection from [{:s}].=====", peer_name);
                        }
                        // else if ext_str == "shtml" { // Dynamic web pages.
                        //     WebServer::respond_with_dynamic_page(stream, path_obj);
                        //     debug!("=====Terminated connection from [{:s}].=====", peer_name);
                        // }
                        else { // Static file request. Dealing with complex queuing, chunk reading, caching...
                            // request scheduling
                            // let file_size = std::io::fs::stat(path_obj).size as uint;
                            let file_size = path_obj.stat().unwrap().size as uint;
                            if file_size < 8000000 {
                                WebServer::respond_with_static_file(cache_arc, path_obj, stream, file_size, file_chunk_size);
                            } else {
                                WebServer::enqueue_static_file_request(stream, path_obj, stream_map_arc, req_queue_arc, notify_chan);
                            }
                        }
                    }
                });
            } // for
        });
    }

    fn respond_with_default_page(stream: Option<std::io::net::tcp::TcpStream>, visitor_count_arc: Arc<RWLock<uint>>) {
        //let visitor_count_arc = self.visitor_count_arc.clone();
        let mut stream = stream.unwrap();
        let response: ~str =
            format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
             <doctype !html><html><head><title>Hello, Rust!</title>
             <style>body \\{ background-color: \\#111; color: \\#FFEEAA \\}
                    h1 \\{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red\\}
                    h2 \\{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green\\}
             </style></head>
             <body>
             <h1>Greetings, Krusty!</h1>
             <h2>Visitor count: {0:u}</h2>
             </body></html>\r\n", *(visitor_count_arc.clone().read()));
        stream.write(response.as_bytes());
    }

    // fn respond_with_dynamic_page(stream: Option<std::io::net::tcp::TcpStream>, path_obj: &Path) {
    //     let mut stream = stream;
    //     let contents = File::open(path_obj).read_to_str();
    //     stream.write("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n".as_bytes());
    //     // TODO: improve the parsing code.
    //     for line in contents.lines() {
    //         if line.contains("<!--#exec cmd=\"") {
    //             let start = line.find_str("<!--#exec cmd=\"").unwrap();
    //             let start_cmd = start + 15;
    //             let mut end_cmd = -1;
    //             let mut end = -1;
    //             for i in range(start_cmd+1, line.len()) {
    //                 if line.char_at(i) == '"' {
    //                     end_cmd = i;
    //                 } else if line.char_at(i) == '>' {
    //                     end = i + 1;
    //                 }
    //                 if end_cmd != -1 && end != -1 {
    //                     break;
    //                 }
    //             }
    //             if end_cmd == -1 || end == -1 || end_cmd >= end {
    //                 stream.write(line.as_bytes());
    //             } else {
    //                 stream.write(line.slice_to(start).as_bytes());
    //                 let cmd = line.slice(start_cmd, end_cmd);
    //                 let ret_str = gash::run_cmdline(cmd);
    //                 stream.write(ret_str.as_bytes());
    //                 stream.write(line.slice_from(end).as_bytes());
    //             }
    //         } else {
    //             stream.write(line.as_bytes());
    //         }
    //     }
    // }

    // Streaming file, Application-layer caching,
    fn respond_with_static_file(cache_arc: Arc<Mutex<HashMap<~str, Arc<CacheItem>>>>, path: &Path, stream: Option<std::io::net::tcp::TcpStream>, file_size: uint, file_chunk_size: uint) {
        let mut stream = stream.unwrap();
        let path_str = path.as_str().unwrap();

        /* pseudo code for caching

        lookup cache
        if hit {
            write the bytes in cache to stream
        } else {
            start a background task to update the cache: create an invalid cached iteam with status marked as false, read bytes from file, write into cached item, status marked as true.
            read from file in chunks, and write to stream
        }

        // Done: step 1: all cached.
        // TODO: step 2: smart replacing algorithm. (LRU?)

        */

        if file_size > 200000000 {
            // Ignore the caching.
            debug!("Start reading {} in disk.", path_str);
            let mut file_reader = File::open(path).ok().unwrap();
            stream.write("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream; charset=UTF-8\r\n\r\n".as_bytes());

            // streaming file.
            // read_bytes() raises io_error on EOF. Consequently, we should count the remaining bytes ourselves.
            let mut remaining_bytes = file_size;
            while (remaining_bytes >= file_chunk_size) {
                stream.write(file_reader.read_exact(file_chunk_size).ok().unwrap());
                remaining_bytes -= file_chunk_size;
            }
            stream.write(file_reader.read_exact(remaining_bytes).ok().unwrap());
            debug!("Stop reading {} in disk.", path_str);
        } else {
            let mut cache_item_status;
            unsafe {
                cache_item_status = cache_arc.unsafe_access(|cache| {
                    let cache_item_arc_opt = cache.find(&path_str.to_owned());
                    match cache_item_arc_opt {
                        Some(cache_item_arc) => {cache_item_arc.read(|cache_item| {cache_item.status})},
                        None => -1
                    }
                });
            }

            if cache_item_status == 0 {// OK. just write the bytes in cache into stream.
                debug!("Wait for reading {} in cache.", path_str);
                let cache_item_arc = WebServer::get_cache_item_arc(cache_arc, path_str);

                cache_item_arc.read(|cache_item| {
                    debug!("Start reading {} in cache.", cache_item.file_path);
                    stream.write("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream; charset=UTF-8\r\n\r\n".as_bytes());
                    stream.write(cache_item.data);
                    debug!("Finish reading {} in cache.", cache_item.file_path);
                });
                debug!("Response in cache, oh yeah!!!!!!!!!!!!!!!!!!!");

            } else {
                if cache_item_status == -1 { // Not exist.
                    // start a background task to update the cache.
                    WebServer::insert_cache_item(cache_arc.clone(), ~path.clone(), file_size);
                }
                // It doesn't hit in cache, just read from file.
                debug!("Start reading {} in disk.", path_str);
                let mut file_reader = File::open(path).ok().unwrap();
                stream.write("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream; charset=UTF-8\r\n\r\n".as_bytes());

                // streaming file.
                // read_bytes() raises io_error on EOF. Consequently, we should count the remaining bytes ourselves.
                let mut remaining_bytes = file_size;
                while (remaining_bytes >= file_chunk_size) {
                    stream.write(file_reader.read_exact(file_chunk_size).ok().unwrap());
                    remaining_bytes -= file_chunk_size;
                }
                stream.write(file_reader.read_exact(remaining_bytes).ok().unwrap());
                debug!("Finish reading {} in disk.", path_str);
            }
        }
    }

    fn enqueue_static_file_request(stream: Option<std::io::net::tcp::TcpStream>, path_obj: &Path, stream_map_arc: Arc<Mutex<HashMap<~str, Option<std::io::net::tcp::TcpStream>>>>, req_queue_arc: Arc<Mutex<PriorityQueue<HTTP_Request>>>, shared_notify_chan: Sender<()>) {
        // Save stream in hashmap for later response.
        let mut stream = stream;
        let peer_name = WebServer::get_peer_name(&mut stream);
        let (stream_chan, stream_port) = channel();
        stream_chan.send(stream);
        unsafe {
            // Use unsafe method, because TcpStream in Rust 0.9 doesn't have "Freeze" bound.
            stream_map_arc.unsafe_access(|local_stream_map| {
                let stream = stream_port.recv();
                local_stream_map.swap(peer_name.clone(), stream);
            });
        }

        // Get file size.
        // let file_size = std::io::fs::stat(path_obj).size as uint;
        let file_size = path_obj.stat().unwrap().size as uint;
        // Enqueue the HTTP request.
        let req = HTTP_Request{peer_name: peer_name.clone(), path: ~path_obj.clone(), file_size: file_size, priority: file_size};

        let (req_chan, req_port) = channel();
        req_chan.send(req);
        debug!("Waiting for queue mutex.");
        req_queue_arc.access(|local_req_queue| {
            debug!("Got queue mutex lock.");
            let req: HTTP_Request = req_port.recv();
            local_req_queue.push(req);
            debug!("A new request enqueued, now the length of queue is {:u}.", local_req_queue.len());
        });

        shared_notify_chan.send(()); // Send incoming notification to responder.
    }

    fn dequeue_static_file_request(&mut self) {
        let req_queue_get = self.request_queue_arc.clone();
        let stream_map_get = self.stream_map_arc.clone();

        let concurrency_sem = self.concurrency_sem.clone();
        let file_chunk_size = self.file_chunk_size;

        // Port<> could not be sent to another task. So I have to make it as the main task that can access self.notify_port.

        let (request_chan, request_port) = channel();
        loop {
            concurrency_sem.acquire();  // waiting for concurrency semaphore.
            self.notify_port.recv();    // waiting for new request enqueued.

            req_queue_get.access( |req_queue| {
                match req_queue.maybe_pop() { // SRPT queue.
                    None => { /* do nothing */ }
                    Some(req) => {
                        request_chan.send(req);
                        debug!("A new request dequeued, now the length of queue is {:u}.", req_queue.len());
                    }
                }
            });

            let request = request_port.recv();
            //println(format!("serve file: {:?}", request.path));

            // Get stream from hashmap.
            // Use unsafe method, because TcpStream in Rust 0.9 doesn't have "Freeze" bound.
            let (stream_chan, stream_port) = channel();

            unsafe {
                stream_map_get.unsafe_access(|local_stream_map| {
                    let stream = local_stream_map.pop(&request.peer_name).expect("no option tcpstream");
                    stream_chan.send(stream);
                });
            }

            // Spawn several tasks to respond the requests concurrently.
            let child_concurrency_sem = concurrency_sem.clone();
            let cache_arc = self.cache_arc.clone();
            spawn (proc(){
                let stream = stream_port.recv();
                // Respond with file content.
                WebServer::respond_with_static_file(cache_arc, request.path, stream, request.file_size, file_chunk_size);
                // Close stream automatically.
                debug!("=====Terminated connection from [{:s}].=====", request.peer_name);
                child_concurrency_sem.release();
            });
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

    fn get_cache_item_arc(cache_arc: Arc<Mutex<HashMap<~str, Arc<CacheItem>>>>, path_str: &str) -> Arc<CacheItem> {
        let (cache_item_arc_chan, cache_item_arc_port) = channel();
        unsafe {
            cache_arc.unsafe_access(|cache| {
                let cache_item_arc_opt = cache.find(&path_str.to_owned());
                match cache_item_arc_opt {
                    Some(cache_item_arc) => {cache_item_arc_chan.send(cache_item_arc.clone());},
                    None => {println("error...");}
                }
            });
        }
        cache_item_arc_port.recv()
    }

    fn insert_cache_item(cache_arc: Arc<Mutex<HashMap<~str, Arc<CacheItem>>>>, path: ~Path, file_size: uint) {
        let cache_arc = cache_arc.clone();
        let path_str = path.as_str().expect("invalid path?").to_owned();

        spawn (proc(){
            // insert a cached item with status UPDATING, so that other tasks will just ignore it.
            // then update the cached item, and set the status as OK.

            let mut to_be_updated = false;
            unsafe {
                cache_arc.unsafe_access(|cache| {
                    if cache.find(&path_str.to_owned()).is_none() {
                        let inited_cache_item = CacheItem {
                            file_path: path_str.to_owned(),
                            data: ~[],
                            file_size: file_size,
                            status: 1, //0: OK, 1: UPDATING
                        };
                        cache.insert(path_str.to_owned(), Arc::new(inited_cache_item));
                        to_be_updated = true;
                    } else { // just exit, since other task is updating it.
                        to_be_updated = false;
                    }
                });
            }

            if to_be_updated == true {
                // read the file bytes into memory, then copy it to cache item.
                // read the data out of the cache_arc, so that other tasks can understand the status, not just waiting.
                let mut file_reader = File::open(path).expect("invalid file!");
                let file_data = file_reader.read_to_end();

                let (file_data_chan, file_data_port) = channel();
                file_data_chan.send(file_data);

                let cache_item_arc = WebServer::get_cache_item_arc(cache_arc, path_str);

                cache_item_arc.write(|cache_item| {
                    cache_item.data = file_data_port.recv();
                    cache_item.status = 0;
                });
            }
        }); // do spawn for updating catch on the background.
    }
}

fn get_args() -> (~str, uint, ~str, uint, uint
) {
    fn print_usage(program: &str) {
        println!("Usage: {:s} [options]", program);
        println!("--ip     \tIP address, \"{:s}\" by default.", IP);
        println!("--port   \tport number, \"{:u}\" by default.", PORT);
        println!("--www    \tworking directory, \"{:s}\" by default", WWW_DIR);
        println!("--concurrency\t The max concurrency of responding tasks, \"{:u}\" by default", MAX_CONCURRENCY);
        println!("--chunk-size\t The chunk size for streaming file, \"{:u}\" by default", CHUNK_SIZE);
        println("-h --help \tUsage");
    }

    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();
    let program = args[0].clone();

    let opts = ~[
        getopts::optopt("ip"),
        getopts::optopt("port"),
        getopts::optopt("www"),
        getopts::optopt("concurrency"),
        getopts::optopt("chunk-size"),
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
                 } else { IP.to_owned() };

    let port:uint = if matches.opt_present("port") {
                        from_str::from_str(matches.opt_str("port").expect("invalid port number?")).expect("not uint?")
                    } else { PORT };

    let www_dir_str = if matches.opt_present("www") {
                        matches.opt_str("www").expect("invalid www argument?")
                      } else { WWW_DIR.to_owned() };

    let concurrency:uint = if matches.opt_present("concurrency") {
                            from_str::from_str(matches.opt_str("concurrency").expect("invalid concurrency argument?")).expect("not uint?")
                          } else { MAX_CONCURRENCY };

    let chunk_size:uint = if matches.opt_present("chunk-size") {
                            from_str::from_str(matches.opt_str("chunk-size").expect("invalid chunk-size argument?")).expect("not uint?")
                          } else { CHUNK_SIZE };
    (ip_str, port, www_dir_str, concurrency, chunk_size)
}

fn main() {
    let (ip_str, port, www_dir_str, concurrency, chunk_size) = get_args();
    let mut zhtta = WebServer::new(ip_str, port, www_dir_str, concurrency, chunk_size);
    zhtta.run();
}