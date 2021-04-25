use std::net::UdpSocket;
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
    id: Vec<u64>,
    data: Vec<u8>,
}

///Struct representing a request for data chunks
///
///filename: String, String representing which file to pull from
///chunks: Vec<u64>, Vector of chunks to pull from the file in interval notation (start, end, start, end)
struct ChunkTransaction {
    target: std::net::SocketAddr,
    filename: String,
    starts: Vec<u64>,
    ends: Vec<u64>,
}



///Take a u64 and pack it into an owned array of u8
///Endian agnostic, flexible for a variety of internal representations
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
    transactions: &mut Vec<ChunkTransaction>,
    buffer: [u8; 512],
    id_counter: &mut u64) {

    //Initialize a packet
    let p: UdpTransferPacket = UdpTransferPacket {
        id: Vec::with_capacity(ID_FIELDS),
        data: Vec::with_capacity(BUFFER_SIZE),
    };
    //Parse the packet id fields
    for i in 0..ID_FIELDS {
        p.id[i] = unpack_u8arr_into_u64(&buffer[i*8..(i*8)+8]);
    }
    //parse the packet's data
    p.data.copy_from_slice(&buffer[ID_FIELDS*8..bytes]);

    //TODO finish all server inbound cases
    //A few possible cases
    //(0,0) This is going to be a plaintext request, I give no fucks about security, filename, zeroes, chunks
    if p.id[0] == 0 && p.id[1] == 0 {
        //Parse a filename and chunk requests
        //Null terminated string
        //Find the first null, everything before it is filename, everything after is beginning:end pairs
        let divider = p.data.iter().find(|&&x| x == 0);
    }
    //(nonzero,nonzero): Use the key at index, here is its nonce and a bunch of chunk requests
    //(0,x): differing behavior
    //(0,1) Send me your public key, here is mine
    //(0,2) Associate this key with a file
    //(0,3+n) where n is an integer>=0 and 3+n fits in a 64 bit unsigned: Lets start a slow request, I'm passing a filename, key and chunk request
}

//We have a queue of requests to work with
fn server_send_all_chunks(t: &mut ChunkTransaction, socket: &mut UdpSocket) {
    //Look at this chunk request, send all of its chunks
    
}

fn serve() -> std::io::Result<()> {
    let mut transactions: Vec<ChunkTransaction> = Vec::new();
    let mut server_socket = UdpSocket::bind("127.0.0.1:9001")?;
    server_socket.set_nonblocking(true)?;
    let mut buffer = [0; PACKET_SIZE];
    let mut id_counter: u64 = 1;
    loop {
        //Handle received packets
        match server_socket.recv_from(&mut buffer) {
            Ok((b,a)) => {
                let bytes_count = b;
                let source_address = a;
                server_handle_inbound(bytes_count, source_address, &mut transactions, buffer, &mut id_counter);
            },
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::WouldBlock => continue,
                    _ => return Err(e),
                }
            }
        }
        //And service the transaction queue
        for mut val in transactions.iter_mut() {
            server_send_all_chunks(&mut val, &mut server_socket);
        }
    }
}

//Basic UDP file transfer server
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        //Disregard whatever was passed, start a server
        //Serve files indefinitely until an error happens
        let result = serve();
        return result
    } else if args.len() == 4 {
        //Run in client mode
        //Parse a filename, a port, an address
        //Perform the client portion of transfer
        Ok(())
    } else {
        println!("Server mode:\nbasic_udp\nClient mode:\nbasic_udp <address> <port> <filename>");
        Ok(())
    }
}
