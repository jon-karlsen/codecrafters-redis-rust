use std::{env, net::TcpListener, sync::{Arc, Mutex}};
use redis_starter_rust::ThreadPool;
use resp::connection::{handle_connection, AppState};


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
    let state    = Arc::new( Mutex::new( AppState::default() ) );

    for stream in listener.incoming() {
        match stream {
            Ok( stream ) => {
                let mut stream_copy = stream.try_clone().unwrap();
                let     state       = state.clone();

                pool.execute( move || {
                    let _ = handle_connection( &mut stream_copy, &state );
                });
            }
            Err( e ) => {
                println!( "error: {}", e );
            }
        }
    }
}
