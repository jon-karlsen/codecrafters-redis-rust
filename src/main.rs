use std::{io::{BufWriter, Write}, net::{TcpListener, TcpStream}};
use redis_starter_rust::ThreadPool;
use resp::connection::Resp;


mod resp;


const ADDRESS: &str    = "127.0.0.1";
const PORT   : &str    = "6379";
const PONG   : &[ u8 ] = b"+PONG\r\n";


fn main() {
    let pool     = ThreadPool::new( 4 );
    let listener = TcpListener::bind( format!( "{}:{}", ADDRESS, PORT ) ).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok( stream ) => {
                let mut stream_copy = stream.try_clone().unwrap();

                pool.execute( move || {
                    let _ = Resp::handle_connection( &mut stream_copy );
                });
            }
            Err( e ) => {
                println!( "error: {}", e );
            }
        }
    }
}
