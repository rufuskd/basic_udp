mod range_tree;

use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::Seek;
use std::fs;
use std::fs::File;
use std::mem;
use std::convert::TryInto;
use std::net::UdpSocket;
use std::collections::VecDeque;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use range_tree::RangeTree;
use openssl::symm::{encrypt, decrypt, Cipher};

//Constants defining internal behavior
///Starting out with 512 byte packets
///All packets are made using these two variables
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
///Endian agnostic, big endian is used as network order
pub fn pack_u64_into_u8arr(val: u64) -> [u8; 8] {
    //Take a 64 bit integer, pull the byte values into a vector
    val.to_be_bytes()
}

///Take a slice of a u8 and return a u64
///Size safe, flexible and a good counterpart to pack_u64_into_u8arr
pub fn unpack_u8arr_into_u64(val: &[u8]) -> u64 {
    //Iterate through the value byte vector backwards and multiply/add
    let (bytes,_) = val.split_at(std::mem::size_of::<u64>());
    u64::from_be_bytes(bytes.try_into().unwrap())
}

///Turn an inbound request into a metadata transaction and add it to the server's transacton queue
pub fn add_metadata_transaction(data: &[u8], source: std::net::SocketAddr, transactions: &mut VecDeque<ChunkTransaction>) {
    //Parse a filename and chunk requests
    //First grab the u8 representing filename length
    let namelen: usize = data[0] as usize;
    let filename = String::from_utf8_lossy(&data[1..1+namelen]);
    
    println!("Metadata request received for {}", filename);
    //Now populate the chunk starts and ends (Both are empty for a metadata request)
    let new_transaction = ChunkTransaction {
        filename: String::from(filename),
        target: source,
        starts: VecDeque::new(),
        ends: VecDeque::new(),
    };

    transactions.push_back(new_transaction);
}

///Turn an inbound request into a chunk transaction and add it to the server's transacton queue
pub fn add_chunk_transaction(data: &[u8], source: std::net::SocketAddr, transactions: &mut VecDeque<ChunkTransaction>) {
    //Parse a filename and chunk requests
    //First grab the u8 representing filename length
    let mut byte_counter = 0;
    let namelen: usize = data[byte_counter] as usize;
    byte_counter+=1;
    let filename = String::from_utf8_lossy(&data[byte_counter..byte_counter+namelen]);
    byte_counter+=namelen;

    let interval_count: u64 = unpack_u8arr_into_u64(&data[byte_counter..byte_counter+8]);
    byte_counter+=8;

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
        new_transaction.starts.push_back(start);
        new_transaction.ends.push_back(end);
    }
    //Push the generated transaction into the main queue
    transactions.push_back(new_transaction);
}

///Handle inbound requests
pub fn server_handle_inbound(
    bytes: usize,
    source: std::net::SocketAddr,
    transactions: &mut VecDeque<ChunkTransaction>,
    buffer: &[u8],
) {
    //Variable used in byte packing
    let mut byte_counter: usize = 0;
    //Get packet ID, this determines what type of request the packet is
    let id: u64 = unpack_u8arr_into_u64(&buffer[byte_counter..byte_counter + 8]);
    byte_counter+=8;

    //0 This is going to be a plaintext metadata request
    if id == 0 {
        add_metadata_transaction(&buffer[byte_counter..bytes], source, transactions);
    } 
    //1 This is going to be a plaintext chunk request
    else if id == 1 {
        add_chunk_transaction(&buffer[byte_counter..bytes], source, transactions);
    }
    //This is some other type of request that isn't implemeneted, output an error
    else {
        println!("Got a request type that isn't implemented yet!");
    }
}


//This function is to service transactions
//THIS IS THE ONLY FUNCTION THAT WILL PASS DATA BACK TO THE CLIENT UNDER ANY CIRCUMSTANCES, THIS IS SECURITY CRITICAL!
///Service the transaction represented by t on the socket provided, using the appropriate whitelist, limited by limiter
///If limiter is 0, no limits!
pub fn server_service_transaction(t: &mut ChunkTransaction, socket: &mut UdpSocket, whitelist: &mut HashSet<String>, limiter: u64) -> std::io::Result<()> {
    let mut sent_counter = 0;
    let mut send_buffer: [u8;PACKET_SIZE] = [0; PACKET_SIZE];
    //Any request for a file that is not on the whitelist gets an all zeros response
    if !whitelist.contains(&t.filename) {
        let nil_reply: [u8;PACKET_SIZE] = [0; PACKET_SIZE];
        //Send the packet
        match socket.send_to(&nil_reply[0..PACKET_SIZE],t.target)
        {
            Ok(_) => {},
            Err(e) => {
                println!("Unable to send data to {:?}.  Error:{:?}",t.target,e);
                return Err(e)
            }
        }
    }
    //This is either a metadata request, or a chunk request
    if t.starts.len() == 0{

        let bytes_to_send = metadata_response_packet(&t.filename, &mut send_buffer);
        //Send the packet
        match socket.send_to(&send_buffer[0..bytes_to_send],t.target)
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
            for i in 0..(*e-*s)+1{
                file.seek(SeekFrom::Start((*s+i)*(BUFFER_SIZE as u64)))?;
                let bytes_read = file.read(&mut buffer)?;
                if bytes_read == 0 {
                    println!("A WE FOUND IT");
                    break;
                }
                //Data is ready, put it in a buffer
                let mut packet_buffer: [u8;PACKET_SIZE] = [0; PACKET_SIZE];
                //Starting simple, just unencrypted chunks
                let chunknum = pack_u64_into_u8arr(*s+i);

                //Pack the chunknum into the packet buffer
                let mut byte_counter: usize = 0;
                for byte in chunknum.iter(){
                    packet_buffer[byte_counter] = *byte;
                    byte_counter+=1;
                }
                //Now pack the actual bytes of the chunk
                for byte in buffer[0..bytes_read].iter(){
                    packet_buffer[byte_counter] = *byte;
                    byte_counter+=1;
                }
                //Send the packet, this will loop and another will be sent
                match socket.send_to(&packet_buffer[0..byte_counter],t.target)
                {
                    Ok(_) => {
                        sent_counter += 1;
                        if limiter != 0 && sent_counter >= limiter {
                            //println!("Sent {:?} packets",sent_counter);
                            return Ok(())
                        }
                    },
                    Err(e) => {
                        println!("Unable to send data to {:?}.  Error:{:?}",t.target,e);
                        return Err(e)
                    }
                }
            }
        }
    }

    //println!("Sent {:?} packets",sent_counter);
    Ok(())
}

/// This function sets up a nonblocking UDP server on a provided address serving files on the provided whitelist
/// this is also thread friendly!  Since we're talking UDP, multiple threads can work with the same socket no biggie
pub fn serve(bind_address: &String, whitelist_filename: &String) -> std::io::Result<()> {
    let mut whitelist: HashSet<String> = HashSet::new();
    match File::open(whitelist_filename) {
        Ok(f) => {
            let reader = io::BufReader::new(f);
            for line in reader.lines() {
                if let Ok(item) = line {
                    whitelist.insert(item);
                }
            }
        },
        Err(_) => {

        }
    }

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

    let mut buffer = [0; PACKET_SIZE]; //Need a buffer that can hold our maximum packet size
    loop {
        //Handle received packets
        match server_socket.recv_from(&mut buffer) {
            Ok((bytes_received, address)) => {
                server_handle_inbound(
                    bytes_received,
                    address,
                    &mut transactions,
                    &buffer[0..bytes_received],
                );
            }
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => continue,
                _ => return Err(e),
            },
        }

        //And service the transaction queue
        for mut t in transactions.iter_mut() {
            match server_service_transaction(&mut t, &mut server_socket, &mut whitelist, 0) {
                Ok(_) => {},
                Err(_) => println!("Error sending chunks for {:?}", t.filename),
            }
        }
        transactions.clear();
    }
}

/// Request a file by requesting all of its chunks sequentially, limiting the amount of the file stored in RAM at any moment
pub fn client_request_sequential_limited(target: &String, filename: &String, outfilename: &String, chunk_mem_limit: usize) -> std::io::Result<()> {
    let server_socket: UdpSocket; //Create the socket using provided params
    let mut send_buffer: [u8; PACKET_SIZE] = [0; PACKET_SIZE];
    let mut recv_buffer: [u8; PACKET_SIZE] = [0; PACKET_SIZE];
    let mut outfile = File::create(outfilename)?;


    //Create a key from /dev/urandom
    let mut rand_file = File::open("/dev/urandom").unwrap();
    let mut key_buf = [0u8; 32];
    let mut iv = [0u8; 16];
    rand_file.read_exact(&mut key_buf).unwrap();
    rand_file.read_exact(&mut iv).unwrap();
    println!("We have a key {:?}",key_buf);
    let cipher = Cipher::aes_256_cbc();
    let data = b"SMH Head";
    let ciphertext = encrypt(cipher,&key_buf,Some(&iv),data).unwrap();
    println!("Encrypted text: {:?}",ciphertext);


    let plaintext = decrypt(
        cipher,
        &key_buf[..],
        Some(&iv[..]),
        &ciphertext[..]).unwrap();
    println!("Unencrypts to: {:?}",plaintext.bytes());

    //Bind our socket locally to any available port, this is an outbound request
    match UdpSocket::bind("0.0.0.0:0")
    {
        Ok(s) => server_socket = s,
        Err(e) => {
            println!("Unable to bind a UDP socket. Error:{:?}",e);
            return Err(e);
        }
    }
    //Set to nonblocking
    match server_socket.set_nonblocking(true)
    {
        Ok(_) => {},
        Err(e) => {
            println!("Unable to set nonblocking, error: {:?}",e);
            return Err(e)
        }
    }


    //GOOD, this method handles repeating requests in a reasonable timeframe
    match client_request_metadata(&server_socket, &mut send_buffer, &mut recv_buffer, &target, &filename) {
        Ok(_) => {
            //We're good to go
        },
        Err(e) => {
            println!("Unable to request metadata");
            return Err(e);
        }
    }

    
    //We don't care about the ID field of the returned metadata packet yet TODO
    let chunk_count = unpack_u8arr_into_u64(&recv_buffer[8..16]); //Metadata requests pass back the chunk count as a u64 in bytes 8-16
    println!("Chunks count {:?}",chunk_count);
    if chunk_count == 0 {
        println!("Either the requested file was empty or not on the whitelist of requestable files");
        return Ok(())
    }

    let mut chunk_vector: Vec<Vec<u8>> = Vec::with_capacity(chunk_mem_limit as usize); //Vector used to buffer chunks to be written into the output file
    for _ in 0..chunk_mem_limit {
        chunk_vector.push(Vec::new());
    }
    let mut progress = 0; //Track how many chunks of chunks we have pulled
    let mut part_start: u64 = progress*chunk_mem_limit as u64;
    let mut part_end: u64;
    if (part_start+chunk_mem_limit as u64) < chunk_count {
        part_end = part_start+(chunk_mem_limit-1) as u64;
    } else {
        part_end = chunk_count-1;
    }

    let mut rt: RangeTree = RangeTree::new(part_start as usize,part_end as usize);
    
    let mut counter: Instant = Instant::now(); //Counter used to track how long it has been since we heard anything from the server
    let mut next: bool = true; //Boolean used to indicate that regardless of the counter, it's time to request a new packet
    //So begins the loop, request from progress*memlimit through the lesser of progress+memlimit or chunkcount
    loop {
        //Check a timer and flag to decide if we need to send a request
        match Instant::now().checked_duration_since(counter) {
            Some(diff) => {
                //If we have gone 100 milliseconds without receiving anything, request something
                if next || diff > Duration::from_millis(200) {
                    let mut s: Vec<u64> = Vec::new();
                    let mut e: Vec<u64> = Vec::new();
                    
                    for xint in rt.intervals.iter() {
                        s.push(rt.tree_vec[*xint].start as u64);
                        e.push(rt.tree_vec[*xint].end as u64);
                    }
                    let bytes_to_send = range_chunk_request_packet(filename,s,e,&mut send_buffer);
                    match server_socket.send_to(&send_buffer[0..bytes_to_send], target)
                    {
                        Ok(_) => {
                            if next {
                                //println!("Sent a packet due to nexting");
                            } else {
                                //println!("Sent a packet due to timeout");
                            }
                            
                            counter = Instant::now();
                        },
                        Err(e) => {
                            println!("Unable to send data to {:?}.  Error: {:?}",target, e);
                            return Err(e)
                        }
                    }
                    next = false;
                }
            },
            None => {
                counter = Instant::now();
            }
        }

        match server_socket.recv(&mut recv_buffer)
        {
            //We either get the next packet, miss a packet, or a latecomer arrives
            Ok(br) => {
                let mut chunkdex = unpack_u8arr_into_u64(&recv_buffer[0..8]);
                rt.add_packet(chunkdex as usize);
                //println!("Got a chunk: {:?}", chunkdex);
                if chunkdex >= part_start {
                    chunkdex -= part_start;
                    //println!("And now chunkdex is this {:?}",chunkdex);
                    //Nailed it, got a chunk
                    counter = Instant::now();
                    for byte in &recv_buffer[8..br] {
                        chunk_vector[chunkdex as usize].push(*byte);
                    }
                    
                }
                else { }
            },
            Err(err) => match err.kind() {
                io::ErrorKind::WouldBlock => {},
                _ => return Err(err),
            }
        }

        if rt.intervals.len() == 0{
            //We're done with this bit, increment progress and move on
            //Increment progress and proceed
            for chunk in chunk_vector.iter() {
                outfile.write_all(&chunk[..])?;
            }
            progress+=1;

            if part_end == chunk_count-1 {
                //We're done!
                break;
            } else {
                //Reinitialize all of our data structures
                //Set the new start and end
                part_start = progress*chunk_mem_limit as u64;
                if (part_start+chunk_mem_limit as u64) < chunk_count {
                    part_end = part_start+ (chunk_mem_limit-1) as u64;
                } else {
                    part_end = chunk_count-1;
                }
                //Reinitialize the chunk vector
                for i in 0..chunk_mem_limit {
                    chunk_vector[i].clear();
                }
                next = true;
                
                rt.reinit(part_start as usize, part_end as usize);
            }
        }
    }

    Ok(())
}

///Populates a given send buffer with the necessary fields to request metadata for file with name fname, returns how many bytes are in the packet
pub fn metadata_request_packet(fname: &String, buffer: &mut[u8; PACKET_SIZE]) -> usize {
    let mut byte_counter: usize = 0;
    //Request metadata, ID field of 0
    for byte in pack_u64_into_u8arr(0).iter(){
        buffer[byte_counter] = *byte;
        byte_counter+=1;
    }

    buffer[byte_counter] = fname.len() as u8;
    byte_counter+=1;

    for byte in fname.bytes(){
        buffer[byte_counter] = byte;
        byte_counter+=1;
    }
    
    byte_counter
}

pub fn metadata_response_packet(filename: &String, buffer: &mut[u8;PACKET_SIZE]) -> usize {
    let filesize: u64;
    match fs::metadata(filename) {
        Ok(m) => {
            if m.len() % (BUFFER_SIZE as u64) == 0{
                filesize = m.len()/(BUFFER_SIZE as u64);
            } else {
                filesize = 1+m.len()/(BUFFER_SIZE as u64);
            }
            
            println!("File {:?} found!",filename)
        }
        Err(_) => {
            filesize = 0;
            println!("File {:?} not found",filename)
        }
    }

    //Metadata is ready, put it in the buffer
    let mut byte_counter: usize = 0;
    let id1 = pack_u64_into_u8arr(0);
    let chunk_count = pack_u64_into_u8arr(filesize);
    for byte in id1.iter(){
        buffer[byte_counter] = *byte;
        byte_counter+=1;
    }

    for byte in chunk_count.iter(){
        buffer[byte_counter] = *byte;
        byte_counter+=1;
    }

    byte_counter
}

pub fn client_request_metadata(server_socket: &UdpSocket ,send_buffer: &mut[u8;PACKET_SIZE], recv_buffer: &mut[u8;PACKET_SIZE],target: &String, filename: &String) -> std::io::Result<()> {
    //Send a metadata request until we have a confirmed response or an error
    //Request metadata
    let bytes_to_send = metadata_request_packet(filename, send_buffer);
    match server_socket.send_to(&send_buffer[0..bytes_to_send], target)
    {
        Ok(_) => {},
        Err(e) => {
            println!("Unable to send data to {:?}.  Error: {:?}",target, e);
            return Err(e)
        }
    }

    let mut counter: Instant = Instant::now();
    //Receive metadata
    loop
    {
        match server_socket.recv(&mut recv_buffer[..])
        {
            Ok(_) => { return Ok(()) },
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => { },
                _ => return Err(e),
            }
        }
        match Instant::now().checked_duration_since(counter) {
            Some(diff) => {
                if diff > Duration::from_millis(100) {
                    match server_socket.send_to(&send_buffer[0..bytes_to_send], target)
                    {
                        Ok(_) => {},
                        Err(e) => {
                            println!("Unable to send data to {:?}.  Error: {:?}",target, e);
                            return Err(e)
                        }
                    }
                }
            },
            None => {
                counter = Instant::now();
            }
        }
    }
}


/// Populates a given send buffer with the necessary field to request chunks of a file with name fname, returns how many bytes are in the packet
pub fn range_chunk_request_packet(fname: &String, starts: Vec<u64>, ends: Vec<u64>, send_buffer: &mut[u8; PACKET_SIZE]) -> usize {
    //Variable used in byte packing and overall size determination
    let mut byte_counter = 0;
    //Request a set of chunks (ID of 1)
    for byte in pack_u64_into_u8arr(1).iter(){
        send_buffer[byte_counter] = *byte;
        byte_counter+=1;
    }

    //Length of file name
    send_buffer[byte_counter] = fname.len() as u8;
    byte_counter+=1;

    //The actual filename
    for byte in fname.bytes(){
        send_buffer[byte_counter] = byte;
        byte_counter+=1;
    }

    //How many chunk ranges?  The min of how many fit and how many are requested
    let desired_chunks = starts.len() as u64;
    let fittable_chunks = ((PACKET_SIZE - byte_counter)/(2*mem::size_of::<u64>())) as u64;
    let actual_chunks: u64;
    if desired_chunks < fittable_chunks {
        actual_chunks = desired_chunks;
    } else {
        actual_chunks = fittable_chunks;
    }
    
    for byte in &pack_u64_into_u8arr(actual_chunks){
        send_buffer[byte_counter] = *byte;
        byte_counter+=1;
    }

    //Pack each chunk range
    for it in starts[0..actual_chunks as usize].iter().zip(ends[0..actual_chunks as usize].iter()) {
        let (s,e) = it;
        for byte in &pack_u64_into_u8arr(*s){
            send_buffer[byte_counter] = *byte;
            byte_counter+=1;
        }

        //+1 to get the next chunk
        for byte in &pack_u64_into_u8arr(*e){
            send_buffer[byte_counter] = *byte;
            byte_counter+=1;
        }
    }

    byte_counter
}