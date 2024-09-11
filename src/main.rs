use std::{env, io::Write, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}};
use infra::app_state::{AppState, ServerRole};
use redis_starter_rust::ThreadPool;
use resp::{connection::handle_connection, encode::encode_resp_arr};


mod infra;
mod resp;


const ADDRESS      : &str = "127.0.0.1";
const ARG_PORT     : &str = "--port";
const ARG_REPLICAOF: &str = "--replicaof";


fn main() -> Result<() , Box<dyn std::error::Error>> {
    let     args  = env::args().collect::<Vec<String>>();
    let mut state = AppState::default();

    println!( "args: {:?}", args );

    for ( i, arg ) in args.iter().enumerate() {
        match arg.as_str() {
            ARG_PORT => {
                let port = i32::from_str_radix( &args[ i + 1 ], 10 ).unwrap();
                state.port = port;
            }

            ARG_REPLICAOF => {
                let host_port = &args[ i + 1 ].split_once( " " ).ok_or( "invalid --replicaof arg" )?;
                let host = host_port.0;
                let port = host_port.1;

                state.replication_info.role = ServerRole::Slave;

                let mut master_stream = TcpStream::connect( format!( "{}:{}", host, port ) )?;
                let     message       = encode_resp_arr( vec![ "PING".to_string() ] )?;

                master_stream.write_all( message.as_bytes() )?;
                master_stream.flush()?;
            }

            a => {
                println!( "unknown arg: {}", a );
            }
        }

    }

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
                println!( "error: {}", e );
            }
        }
    };

    Ok( () )
}
