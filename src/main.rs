use basic_udp;

use std::env;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::prelude::*;


//Basic UDP file transfer server
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        //Disregard whatever was passed, start a server
        //Serve files indefinitely until an error happens
        let mut server_arg_map: HashMap<String,String> = HashMap::new();

        match File::open(&args[1]) {
            Ok(config_file) => {
                let reader = io::BufReader::new(config_file);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            let split: Vec<&str> = l.split_ascii_whitespace().collect();
                            if split.len() >= 2{
                                server_arg_map.entry(split[0].to_string()).or_insert(split[1].to_string());
                            }
                        },
                        Err(_) => {
                            //No big deal, go with default settings/behavior
                        }
                    }
                }
            },
            Err(e) => {
                println!("Unable to open config file");
                return Err(e);
            }
        }
        

        //println!("Here's the config map {:?}",server_arg_map);
        //We've parsed the config file, use it or fill out defaults
        if !server_arg_map.contains_key("ip") {
            println!("ip not found in config file, using default IP/port 127.0.0.1:9001");
            server_arg_map.insert(String::from("ip"),String::from("127.0.0.1:9001"));
        }
        if !server_arg_map.contains_key("whitelist") {
            println!("whitelist not found in config file, using default called: whitelist");
            server_arg_map.insert(String::from("whitelist"),String::from("whitelist"));
        }

        let result = basic_udp::serve(&server_arg_map["ip"],&server_arg_map["whitelist"]);

        return result;
    } else if args.len() == 4 {
        //Run in client mode
        //Parse a filename, a port:address
        //Perform the client portion of transfer
        let result = basic_udp::client_request_sequential_limited(&args[1], &args[2], &args[3],1000);
        return result;
    } else {
        println!("Server mode:\nbasic_udp <config file>\nClient mode:\nbasic_udp <address:port> <filename> <outfilename>");
        Ok(())
    }
}
