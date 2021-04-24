use std::net::UdpSocket;
use std::collections::HashSet;
use std::collections::HashMap;
use std::io;
use std::env;
use std::mem;

//Starting out with 512 byte packets
const PACKET_SIZE: usize = 512;
//The amount of identifying fields in a packet
const ID_FIELDS: usize = 2;
const BUFFER_SIZE: usize = PACKET_SIZE-(ID_FIELDS*mem::size_of::<u64>());

///Struct representing a packet
/// 
///id: [u64;ID_FIELDS],
///data: [u8;BUFFER_SIZE],
struct UdpTransferPacket {
    id: [u64;ID_FIELDS],
    data: [u8;BUFFER_SIZE],
}

///Take a u64 and pack it into an owned array of u8
fn pack_u64_into_u8arr(val: u64) -> [u8;8] {
    //Take a 64 bit integer, pull the byte values into a vector
    let mut working_val = val;
    let mut retval: [u8;8] = [0;8];
    for i in 0..8 {
        retval[i] = (working_val%256) as u8;
        working_val = working_val/256;
    }

    retval
}

//Take a slice of a u8 and return a u64
fn unpack_u8arr_into_u64(val: &[u8]) -> u64 {
    //Iterate through the value byte vector backwards and multiply/add
    if val.len() > 8 {
        println!("Can't do it, max byte vector we can convert to u64 is 8 bytes");
        0
    } else {
        let mut result: u64 = 0;
        for i in (0..val.len()).rev() {
            result = result*256;
            result += val[i] as u64;
        }

        result
    }
}

fn server_handle_inbound(
    bytes: usize,
    source: std::net::SocketAddr,
    buffer: [u8; 512],) {

    //Initialize a packet
    let p: UdpTransferPacket = UdpTransferPacket {
        id: [0: u64;ID_FIELDS],
        data: [0: u8;BUFFER_SIZE],
    };
    //Parse the packet into it
    p.id[0] = unpack_u8arr_into_u64(&buffer[0..8]);
    p.id[1] = unpack_u8arr_into_u64(&buffer[8..16]);
    p.data.copy_from_slice(&buffer[16..bytes]);

    //TODO finish all server inbound cases
    //A few possible cases
    //(nonzero,nonzero): Use the key at index, here is its nonce and a bunch of chunk requests
    //(0,x): differing behavior
    //(0,0) This is going to be a plaintext request, I give no fucks about security, filename, zeroes, chunks
    //(0,1) Send me your public key, here is mine
    //(0,2) Associate this key with a file
    //(0,3+n) where n is an integer>=0 and 3+n fits in a 64 bit unsigned: Lets start a slow request, I'm passing a filename, key and chunk request
}

//We have a queue of requests to work with and a key map to deal with
//fn server_send_chunks(client: &mut ClientConnection, socket: &mut UdpSocket) {
    //Send file chunks to a client
    //Starting naive, open a file, seek according to client connection params
    //Make the packet, send the chunk on the provided socket
    //Get the id to send back
    //Determine which chunk to send back and store it in a buffer
    //Pull the chunk and store it in a buffer
    //Mash all the buffers into one
    //Send the buffer
//}

fn serve() -> std::io::Result<()> {
    let mut clients: HashMap<u64, ClientConnection> = HashMap::new();
        let server_socket = UdpSocket::bind("127.0.0.1:9001")?;
        server_socket.set_nonblocking(true)?;
        let mut buffer = [0; BUFFER_SIZE];
        let mut id_counter: u64 = 1;
        loop {
            //Handle received packets
            match server_socket.recv_from(&mut buffer) {
                Ok((b,a)) => {
                    let bytes_count = b;
                    let source_address = a;
                    server_handle_inbound(bytes_count, source_address, &mut clients, buffer, &mut id_counter);
                },
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::WouldBlock => continue,
                        _ => return Err(e),
                    }
                }
            }
            //And send out whatever we need
            for val in clients.values_mut() {
                server_send_chunks(val);
            }
        }
}

//Basic UDP file transfer server
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        //Disregard whatever was passed, start a server
        //Serve files indefinitely until an error happens
        let result = serve();
        return result
    } else {
        //Run in client mode
        //Parse a filename, a port, an address
        //Perform the client portion of transfer
        Ok(())
    }
}
