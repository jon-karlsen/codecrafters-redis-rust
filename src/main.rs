use std::{io::{Read, Write}, net::TcpListener};


fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!( "Logs from your program will appear here!" );

     let listener = TcpListener::bind( "127.0.0.1:6379" ).unwrap();

     for stream in listener.incoming() {
         match stream {
             Ok( mut stream ) => {
                    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\n+PONG\r\n";
                    let mut buf  = [ 0; 1024 ];
                    let     _    = stream.read( &mut buf );

                    if String::from_utf8_lossy( &buf ).contains( "PING" ) {
                        let _ = stream.write_all( response ).expect( "write failed" );
                    }

                    stream.flush().expect( "flush failed" );
             }
             Err( e ) => {
                 println!( "error: {}", e );
             }
         }
     }
}
