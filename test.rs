use std::io::println;
fn main() {
    let mut x = ~5;
    if *x < 10 {
        let mut y = &x;
        *y -= 1;
        println!("Oh no:{:?}", y);
    }
    *x -= 1;
    println!("Oh no: {:?}", x);
}
