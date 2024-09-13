use std::{env, net::TcpListener, sync::{Arc, Mutex}};
use infra::app_state::AppState;
use redis_starter_rust::ThreadPool;
use resp::connection::handle_connection;


mod infra;
mod resp;


const ADDRESS : &str = "127.0.0.1";


fn main() -> Result<() , Box<dyn std::error::Error>> {
    let args  = env::args().collect::<Vec<String>>();
    let state = AppState::from_args( &args )?;

    println!( "args: {:?}", args );

    let pool      = ThreadPool::new( 4 );
    let listener  = TcpListener::bind( format!( "{}:{}", ADDRESS, &state.port ) ).unwrap();
    let state_arc = Arc::new( Mutex::new( state ) );

    for stream in listener.incoming() {
        match stream {
            Ok( stream ) => {
                let mut stream_copy = stream.try_clone().unwrap();
                let     state       = state_arc.clone();

                pool.execute( move || {
                    let _ = handle_connection( &mut stream_copy, &state );
                });
            }
            Err( e ) => {
                println!( "error: {:?}", e );
            }
        }
    };

    Ok( () )
}
