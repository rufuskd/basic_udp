use basic_udp;

use std::env;

//Basic UDP file transfer server
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        //Disregard whatever was passed, start a server
        //Serve files indefinitely until an error happens
        let result = basic_udp::serve(&args[1]);
        return result;
    } else if args.len() == 3 {
        //Run in client mode
        //Parse a filename, a port:address
        //Perform the client portion of transfer
        let result = basic_udp::request(&args[1], &args[2]);
        return result;
    } else {
        println!("Server mode:\nbasic_udp\nClient mode:\nbasic_udp <address:port> <filename>");
        Ok(())
    }
}
