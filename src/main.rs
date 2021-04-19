use std::net::UdpSocket;
use std::collections::HashSet;
use std::collections::HashMap;
use std::io;
use std::env;

struct ClientConnection {
    id: u64,
    addr: std::net::SocketAddr,
    filename: String,
    startChunk: u64,
    endChunk: u64,
    ackChunks: HashSet<u64>,
}

struct UdpTransferPacket<'a> {
    id: u64,
    chunk_id: u64,
    data: &'a [u8],
}

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
    client_vector: &mut HashMap<u64, ClientConnection>,
    buffer: [u8; 512],
    id_count: &mut u64) {

    //Parse the packet
    let p: UdpTransferPacket = UdpTransferPacket {
        id: unpack_u8arr_into_u64(&buffer[0..8]),
        chunk_id: unpack_u8arr_into_u64(&buffer[8..16]),
        data: &buffer[16..bytes],
    };

    //A few possible cases
    
    if p.id == 0 {
        //New request - ID of zero, source is arbitrary, buffer contains a filename
        //Create a new client connection, add it to the map
        let new_client = ClientConnection {
            id: *id_count,
            addr: source,
            filename: String::from_utf8_lossy(&p.data).to_string(),
            startChunk: 0,
            endChunk: 0,
            ackChunks: HashSet::new(),
        };
        client_vector.insert(*id_count, new_client);
        *id_count+=1;

        //Now send back an acknowledgment so they know the download has begun

    } else if client_vector.contains_key(&p.id) {
        //Ack for existing client
        //Update the connection in the map, if it's the last ack, clear em out
    } else {
        //Not a new request, not an existing client, naughty naughty
        println!("Received a packet that doesn't match an existing download and isn't a new request");
    }
}

fn server_send_chunks(client: &mut ClientConnection, socket: &mut UdpSocket) {
    //Send file chunks to a client
    struct ClientConnection {
        id: u64,
        addr: std::net::SocketAddr,
        filename: String,
        startChunk: u64,
        endChunk: u64,
        ackChunks: HashSet<u64>,
    }
    //Starting naive, open a file, seek according to client connection params
    //Make the packet, send the chunk on the provided socket
}

fn serve() -> std::io::Result<()> {
    let mut clients: HashMap<u64, ClientConnection> = HashMap::new();
        let server_socket = UdpSocket::bind("127.0.0.1:9001")?;
        server_socket.set_nonblocking(true)?;
        let mut buffer = [0; 512];
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
