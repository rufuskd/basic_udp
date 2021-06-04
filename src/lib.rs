use std::collections::VecDeque;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::mem;
use std::net::UdpSocket;
use std::io::SeekFrom;
use std::io::Seek;

//Constants defining internal behavior
///Starting out with 512 byte packets
const PACKET_SIZE: usize = 512;
///The amount of identifying fields in a packet
const ID_FIELDS: usize = 2;
///Packet size minus fields*size of fields
const BUFFER_SIZE: usize = PACKET_SIZE - (ID_FIELDS * mem::size_of::<u64>());

///Struct representing a packet
///
///id: Vec<u64>,
///data: Vec<u8>,
pub struct UdpTransferPacket {
    id: Vec<u64>,
    data: Vec<u8>,
}

///Struct representing a request for data chunks
///
///filename: String, String representing which file to pull from
///starts: Vec<u64>, Vector of interval beginnings for chunks to pull
///starts: Vec<u64>, Vector of offset endings for chunks to pull
pub struct ChunkTransaction {
    target: std::net::SocketAddr,
    filename: String,
    starts: VecDeque<u64>,
    ends: VecDeque<u64>,
}

///Take a u64 and pack it into an owned array of u8
///Endian agnostic, flexible for a variety of internal representations
pub fn pack_u64_into_u8arr(val: u64) -> [u8; 8] {
    //Take a 64 bit integer, pull the byte values into a vector
    let mut working_val = val;
    let mut retval: [u8; 8] = [0; 8];
    for i in 0..8 {
        retval[i] = (working_val % 256) as u8;
        working_val = working_val / 256;
    }

    retval
}

///Take a slice of a u8 and return a u64
///Size safe, flexible and a good counterpart to pack_u64_into_u8arr
pub fn unpack_u8arr_into_u64(val: &[u8]) -> u64 {
    //Iterate through the value byte vector backwards and multiply/add
    if val.len() > 8 {
        println!("Can't do it, max byte vector we can convert to u64 is 8 bytes");
        0
    } else {
        let mut result: u64 = 0;
        for i in (0..val.len()).rev() {
            result = result * 256;
            result += val[i] as u64;
        }

        result
    }
}


///Handle inbound request for chunks
pub fn server_handle_inbound(
    bytes: usize,
    source: std::net::SocketAddr,
    transactions: &mut VecDeque<ChunkTransaction>,
    buffer: [u8; 512],
) {
    //Initialize a packet
    let mut p: UdpTransferPacket = UdpTransferPacket {
        id: Vec::with_capacity(ID_FIELDS),
        data: Vec::with_capacity(BUFFER_SIZE),
    };
    //Parse the packet id fields
    for i in 0..ID_FIELDS {
        p.id[i] = unpack_u8arr_into_u64(&buffer[i * 8..(i * 8) + 8]);
    }
    //parse the packet's data
    p.data.copy_from_slice(&buffer[ID_FIELDS * 8..bytes]);

    //(0,0) This is going to be a plaintext request, I give no fucks about security, filename, zeroes, chunks
    if p.id[0] == 0 && p.id[1] == 0 {
        //Parse a filename and chunk requests
        //Null terminated string
        //Find the first null, everything before it is filename, everything after is beginning:end pairs
        let divider: usize;

        match p.data.iter().position(|&x| x == 0) {
            Some(x) => divider = x,
            None => {
                println!("Uh oh, couldn't parse a filename, quitting");
                return;
            }
        }

        let filename = String::from_utf8_lossy(&p.data[0..divider]);
        println!("Pulling chunks from {}", filename);

        //Now populate the chunk starts and ends
        let mut new_transaction = ChunkTransaction {
            filename: String::from(filename),
            target: source,
            starts: VecDeque::new(),
            ends: VecDeque::new(),
        };

        //Iterate from divider to the end of p.data
        for i in (divider..bytes).step_by(2 * mem::size_of::<u64>()) {
            new_transaction
                .starts
                .push_back(unpack_u8arr_into_u64(&p.data[i..i + mem::size_of::<u64>()]));
            new_transaction
                .ends
                .push_back(unpack_u8arr_into_u64(&p.data[i..i + mem::size_of::<u64>()]));
            println!(
                "Got a chunk request from {:?} to {:?}",
                new_transaction.starts.back(),
                new_transaction.ends.back()
            );
        }

        //Push the generated transaction into the main queue
        transactions.push_back(new_transaction);
    } else {
        println!("Got a request type that isn't implemented yet!");
    }
}


//We have a queue of requests to work with
pub fn server_send_all_chunks(t: &mut ChunkTransaction, socket: &mut UdpSocket) -> std::io::Result<()> {
    //Look at this chunk request, send all of its chunks
    //If no chunks were requested, then respond with the total
    let mut file = File::open(&t.filename)?;
    
    let mut buffer: [u8;BUFFER_SIZE] = [0; BUFFER_SIZE];
    for it in t.starts.iter().zip(t.ends.iter()) {
        let (s,e) = it;
        
        //file.seek(SeekFrom::Start(*s*(BUFFER_SIZE as u64)))?;
        //iterate from 0 to *e, make a packet and send it
        for i in 0..*e{
            file.seek(SeekFrom::Start((*s+i)*(BUFFER_SIZE as u64)))?;
            file.read_exact(&mut buffer)?;
            //Data is ready, make a packet out of it
            let mut p: UdpTransferPacket = UdpTransferPacket {
                id: Vec::with_capacity(ID_FIELDS),
                data: Vec::with_capacity(BUFFER_SIZE),
            };
            
        }
    }
    

    Ok(())
}

pub fn serve() -> std::io::Result<()> {
    let mut transactions: VecDeque<ChunkTransaction> = VecDeque::new();
    let mut server_socket = UdpSocket::bind("127.0.0.1:9001")?;
    server_socket.set_nonblocking(true)?;
    let mut buffer = [0; PACKET_SIZE];
    loop {
        //Handle received packets
        match server_socket.recv_from(&mut buffer) {
            Ok((b, a)) => {
                let bytes_count = b;
                let source_address = a;
                server_handle_inbound(
                    bytes_count,
                    source_address,
                    &mut transactions,
                    buffer,
                );
            }
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => continue,
                _ => return Err(e),
            },
        }
        //And service the transaction queue
        for mut val in transactions.iter_mut() {
            server_send_all_chunks(&mut val, &mut server_socket);
        }
    }
}

pub fn request(target: &String, filename: &String) -> std::io::Result<()> {
    //Create the socket using provided params
    let mut server_socket = UdpSocket::bind(target)?;
    server_socket.set_nonblocking(true)?;
    let mut buffer = [0; PACKET_SIZE];
    //Request the metadata
    let mut req = UdpTransferPacket {
        id: Vec::with_capacity(ID_FIELDS),
        data: Vec::with_capacity(BUFFER_SIZE),
    };
    req.id[0] = 0;
    req.id[1] = 1;
    req.data = filename.clone().into_bytes();

    //Request chunks until we have them all

    //On receipt of a chunk, store it, then update a chunk ledger

    //Reassemble the downloaded file from its chunks
    Ok(())
}
