use std::{env, net::TcpListener};
use redis_starter_rust::ThreadPool;
use resp::connection::handle_connection;


mod resp;


const ADDRESS: &str = "127.0.0.1";


fn main() {
    let mut port     = 6379;
    let     args     = env::args().collect::<Vec<String>>();

    for ( i, arg ) in args.iter().enumerate() {
        if arg.starts_with( "--port" ) {
            port = i32::from_str_radix( &args[ i + 1 ], 10 ).unwrap();
        }
    }

    let pool     = ThreadPool::new( 4 );
    let listener = TcpListener::bind( format!( "{}:{}", ADDRESS, port ) ).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok( stream ) => {
                let mut stream_copy = stream.try_clone().unwrap();

                pool.execute( move || {
                    let _ = handle_connection( &mut stream_copy );
                });
            }
            Err( e ) => {
                println!( "error: {}", e );
            }
        }
    }
}
