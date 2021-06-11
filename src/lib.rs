use std::collections::VecDeque;
use std::io;
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::mem;
use std::net::UdpSocket;
use std::io::SeekFrom;
use std::io::Seek;

//Constants defining internal behavior
///Starting out with 512 byte packets
const PACKET_SIZE: usize = 512;
const BUFFER_SIZE: usize = PACKET_SIZE - mem::size_of::<u64>();

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
    //Get packet data, starting with the id field
    let mut byte_counter: usize = 0;
    let id: u64 = unpack_u8arr_into_u64(&buffer[byte_counter * 8..((byte_counter * 8) + 8)]);
    byte_counter+=1;

    //Get the packet's data
    let mut data: Vec<u8> = Vec::with_capacity(BUFFER_SIZE);
    for i in byte_counter*8..bytes {
        data.push(buffer[i]);
    }

    //0 This is going to be a plaintext request, I give no fucks about security, filename, zeroes, chunks
    if id == 0 {
        //Parse a filename and chunk requests
        //First grab the u8 representing filename length
        let namelen: usize = data[0] as usize;
        let filename = String::from_utf8_lossy(&data[1..namelen+1]);
        
        println!("Metadata request received for {}", filename);

        //Now populate the chunk starts and ends
        let new_transaction = ChunkTransaction {
            filename: String::from(filename),
            target: source,
            starts: VecDeque::new(),
            ends: VecDeque::new(),
        };

        //Push the generated transaction into the main queue
        transactions.push_back(new_transaction);
    } else if id == 1 {
        //Parse a filename and chunk requests
        //First grab the u8 representing filename length
        let mut byte_counter = 0;
        let namelen: usize = data[byte_counter] as usize;
        byte_counter+=1;
        let filename = String::from_utf8_lossy(&data[byte_counter..byte_counter+namelen]);
        byte_counter+=namelen;
        let interval_count: u64 = unpack_u8arr_into_u64(&data[byte_counter..byte_counter+8]);
        byte_counter+=8;
        
        println!("Chunk request received for {}", filename);

        //Now populate the chunk starts and ends
        let mut new_transaction = ChunkTransaction {
            filename: String::from(filename),
            target: source,
            starts: VecDeque::new(),
            ends: VecDeque::new(),
        };

        for _ in 0..interval_count {
            let start = unpack_u8arr_into_u64(&data[byte_counter..byte_counter+8]);
            byte_counter+=8;
            let end = unpack_u8arr_into_u64(&data[byte_counter..byte_counter+8]);
            byte_counter+=8;
            //println!("Pushing a chunk range start:{:?} end:{:?}",start,end);
            new_transaction.starts.push_back(start);
            new_transaction.ends.push_back(end);
        }
        //Push the generated transaction into the main queue
        transactions.push_back(new_transaction);
    } else {
        println!("Got a request type that isn't implemented yet!");
    }
}


//This function is to service transactions
pub fn server_send_all_chunks(t: &mut ChunkTransaction, socket: &mut UdpSocket) -> std::io::Result<()> {
    
    //This is either a metadata request, or a chunk request
    if t.starts.len() == 0{
        let filesize: u64;
        match fs::metadata(&t.filename) {
            Ok(m) => {
                if m.len() % (BUFFER_SIZE as u64) == 0{
                    filesize = m.len()/(BUFFER_SIZE as u64);
                } else {
                    filesize = 1+m.len()/(BUFFER_SIZE as u64);
                }
                
                println!("File {:?} found!",t.filename)
            }
            Err(_) => {
                filesize = 0;
                println!("File {:?} not found",t.filename)
            }
        }

        //Metadata is ready, put it in a buffer
        let mut packet_buffer: [u8;PACKET_SIZE] = [0; PACKET_SIZE];
        let mut byte_counter: usize = 0;
        let id1 = pack_u64_into_u8arr(0);
        let chunk_count = pack_u64_into_u8arr(filesize);
        for byte in id1.iter(){
            packet_buffer[byte_counter] = *byte;
            byte_counter+=1;
        }

        for byte in chunk_count.iter(){
            packet_buffer[byte_counter] = *byte;
            byte_counter+=1;
        }

        //Send the packet
        match socket.send_to(&packet_buffer,t.target)
        {
            Ok(_) => {},
            Err(e) => {
                println!("Unable to send data to {:?}.  Error:{:?}",t.target,e);
                return Err(e)
            }
        }
    } else {
        let mut file = File::open(&t.filename)?;
        let mut buffer: [u8;BUFFER_SIZE] = [0; BUFFER_SIZE];
        //Look through all requested chunks and grab em
        for it in t.starts.iter().zip(t.ends.iter()) {
            let (s,e) = it;
            file.seek(SeekFrom::Start(*s*(BUFFER_SIZE as u64)))?;
            //iterate from 0 to *e, make a packet and send it
            for i in 0..*e{
                file.seek(SeekFrom::Start((*s+i)*(BUFFER_SIZE as u64)))?;
                //file.read_exact(&mut buffer)?;
                file.read(&mut buffer)?;
                //Data is ready, put it in a buffer
                let mut packet_buffer: [u8;PACKET_SIZE] = [0; PACKET_SIZE];
                //Starting simple, just unencrypted chunks
                let chunknum = pack_u64_into_u8arr((*s+i)*(BUFFER_SIZE as u64));
                //TODO very unsophisticated way of packing these, but good enough for now
                let mut byte_counter: usize = 0;
                for byte in chunknum.iter(){
                    packet_buffer[byte_counter] = *byte;
                    byte_counter+=1;
                }
                for byte in buffer.iter(){
                    packet_buffer[byte_counter] = *byte;
                    byte_counter+=1;
                }
                //Send the packet
                match socket.send_to(&packet_buffer,t.target)
                {
                    Ok(_) => {},
                    Err(e) => {
                        println!("Unable to send data to {:?}.  Error:{:?}",t.target,e);
                        return Err(e)
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn serve(bind_address: &String) -> std::io::Result<()> {
    let mut transactions: VecDeque<ChunkTransaction> = VecDeque::new();
    let mut server_socket: UdpSocket;
    match UdpSocket::bind(bind_address)
    {
        Ok(s) => server_socket = s,
        Err(e) => {
            println!("Unable to bind a UDP socket {:?}. Error:{:?}",bind_address,e);
            return Err(e);
        }
    }
    match server_socket.set_nonblocking(true)
    {
        Ok(_) => {},
        Err(_) => {
            println!("Unable to set nonblocking. Performance will be terrible");
        }
    }

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
            match server_send_all_chunks(&mut val, &mut server_socket) {
                Ok(_) => {},
                Err(_) => println!("Error sending chunks for {:?}", val.filename),
            }
        }
        transactions.clear();
    }
}

pub fn request(target: &String, filename: &String) -> std::io::Result<()> {
    //Create the socket using provided params
    let server_socket: UdpSocket;
    let mut chunk_vector: Vec<u8> = Vec::new();
    match UdpSocket::bind("0.0.0.0:0")
    {
        Ok(s) => server_socket = s,
        Err(e) => {
            println!("Unable to bind a UDP socket. Error:{:?}",e);
            return Err(e);
        }
    }
    match server_socket.set_nonblocking(true)
    {
        Ok(_) => {},
        Err(e) => {
            println!("Unable to set nonblocking, error: {:?}",e);
            return Err(e)
        }
    }
    let mut buffer = [0; PACKET_SIZE];
    //Request the metadata
    let id = pack_u64_into_u8arr(0);
    let fname = filename.clone().into_bytes();
    let fname_length: u8 = fname.len() as u8;
    let mut byte_counter: usize = 0;

    //Request metadata
    for byte in id.iter(){
        buffer[byte_counter] = *byte;
        byte_counter+=1;
    }

    buffer[byte_counter] = fname_length;
    byte_counter+=1;

    for byte in fname.iter(){
        buffer[byte_counter] = *byte;
        byte_counter+=1;
    }
    
    match server_socket.send_to(&buffer, target)
    {
        Ok(_) => {},
        Err(e) => {
            println!("Unable to send data to {:?}.  Error: {:?}",target, e);
            return Err(e)
        }
    }

    //Receive metadata
    loop
    {
        match server_socket.recv(&mut buffer)
        {
            Ok(_) => {break},
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => continue,
                _ => return Err(e),
            }
        }
    }
    
    let chunk_count = unpack_u8arr_into_u64(&buffer[8..16]);
    let mut next_chunk: u64 = 0;
    println!("File length is: {:?} chunks",unpack_u8arr_into_u64(&buffer[8..16]));

    //Request chunks until we have the whole file
    loop {
        byte_counter = 0;
        //Request metadata (ID of 1)
        for byte in pack_u64_into_u8arr(1).iter(){
            buffer[byte_counter] = *byte;
            byte_counter+=1;
        }

        //Length of file
        buffer[byte_counter] = fname_length;
        byte_counter+=1;

        //The actual filename
        for byte in fname.iter(){
            buffer[byte_counter] = *byte;
            byte_counter+=1;
        }

        //Just 1 start/end
        for byte in pack_u64_into_u8arr(1).iter(){
            buffer[byte_counter] = *byte;
            byte_counter+=1;
        }

        //The current chunk is all we're requesting
        for byte in pack_u64_into_u8arr(next_chunk).iter(){
            buffer[byte_counter] = *byte;
            byte_counter+=1;
        }

        //Start and end at the same spot for 1 chunk
        for byte in pack_u64_into_u8arr(next_chunk+1).iter(){
            buffer[byte_counter] = *byte;
            byte_counter+=1;
        }
        
        match server_socket.send_to(&buffer, target)
        {
            Ok(_) => {},
            Err(e) => {
                println!("Unable to send data to {:?}.  Error: {:?}",target, e);
                return Err(e)
            }
        }

        //Receive all chunks one at a time
        let mut counter = 0;
        loop
        {
            match server_socket.recv(&mut buffer)
            {
                Ok(_) => {
                    next_chunk += 1;
                    break;
                },
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock => {
                        if counter < 100000 {
                            counter += 1;
                        } else {
                            break;
                        }
                    },
                    _ => return Err(e),
                }
            }
        }

        for byte in buffer[8..].iter() {
            chunk_vector.push(*byte);
        }
        if next_chunk >= chunk_count{
            break;
        }
    }
    
    //Iterate over the chunk vector and make a file!
    let mut outfile = File::create("Testout")?;
    outfile.write_all(&chunk_vector[..])?;

    Ok(())
}
