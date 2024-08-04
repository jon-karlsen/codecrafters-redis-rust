use std::{io::{BufRead, BufReader, BufWriter, Write}, net::{TcpListener, TcpStream}};


const ADDRESS: &str    = "127.0.0.1";
const PORT   : &str    = "6379";
const PONG   : &[ u8 ] = b"+PONG\r\n";


fn handle_request( stream: &mut TcpStream ) {
    let     stream_clone = stream.try_clone().unwrap();
    let mut reader       = BufReader::new( stream );
    let mut writer       = BufWriter::new( stream_clone );
    let mut buf: Vec<u8> = Vec::new();

    loop {
        let bytes_read = reader.read_until( b'\n', &mut buf ).unwrap();

        if bytes_read == 0 {
            break;
        }

        let line = String::from_utf8( buf.clone() ).unwrap();

        println!( "{}", line );

        if line.trim() == "PING" {
            writer.write_all( PONG ).unwrap();
            writer.flush().unwrap();
        }

        buf.clear();
    }
}


fn main() {
    let listener = TcpListener::bind( format!( "{}:{}", ADDRESS, PORT ) ).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok( mut stream ) => {
                handle_request( &mut stream );
            }
            Err( e ) => {
                println!( "error: {}", e );
            }
        }
    }
}
