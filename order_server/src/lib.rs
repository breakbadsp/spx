use std::{io::{BufReader, Read}, mem::transmute, net::{TcpListener, TcpStream}};
use msg::{order::{self, *}, MsgHeader};

pub fn CreateListener() {
    let listener_thread = std::thread::spawn(|| {
        let listener = TcpListener::bind("127.0.0.1:2700").unwrap();
        println!("Server started listening on port 2700");
        for stream_or_error in listener.incoming() {
            match stream_or_error {
                Ok(stream) => {
                  
                }
                Err(_err) => {
                    println!("Server failed to accept incoming connection.")
                }
            }
        }
    });
}

pub fn OrderReceiver(mut p_stream: TcpStream) {
    loop {
        let mut stream_copy = p_stream.try_clone().unwrap();
        let mut buff_reader = BufReader::new(&mut stream_copy);
        let mut buffer = [0; size_of::<msg::MsgHeader>()];
        buff_reader.read_exact(&mut buffer).unwrap();

        let msg_header_or_error = MsgHeader::from_bytes(&buffer);
        
        let (msg_len, msg_type) = match msg_header_or_error {
            Err(error) => {
                println!("Failed to read the header!");
                continue;
            }
            Ok(msg_header) => {
                (msg_header.msg_len, msg_header.msg_type)
            }
        };

        match msg_type {
            msg::MsgType::Order => {
                
            }

            msg::MsgType::Invalid   => {

            }
        }

    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
