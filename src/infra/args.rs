use std::{io::{Read, Write}, net::TcpStream};

use crate::resp::{constants::{CMD_PSYNC, CMD_REPLCONF}, encode::encode_resp_arr};

use super::app_state::{AppState, ServerRole};


const ARG_PORT     : &str = "--port";
const ARG_REPLICAOF: &str = "--replicaof";


fn handle_arg_port( state: &mut AppState,
                    arg  : &str ) -> Result<(), Box<dyn std::error::Error>> {
    let port = i32::from_str_radix( arg, 10 ).unwrap();
    state.port = port;

    Ok( () )
}


fn handle_arg_replicaof( state: &mut AppState, arg: &str ) -> Result<(), Box<dyn std::error::Error>> {
    let host_port = arg.split_once( " " ).ok_or( "invalid --replicaof arg" )?;
    let host      = host_port.0;
    let port      = host_port.1;

    state.replication_info.role = ServerRole::Slave;

    let mut stream  = TcpStream::connect( format!( "{}:{}", host, port ) )?;
    let     message = encode_resp_arr( vec![ "PING".to_string() ] )?;

    stream.write_all( message.as_bytes() )?;
    stream.flush()?;

    let mut buffer  = [ 0; 1024 ];
    let mut counter = 0;

    loop {
        match stream.read( &mut buffer ) {
            Ok( 0 ) => {
                break;
            }

            Ok( bytes_read ) => {
                let res = &buffer[ ..bytes_read ];

                if counter == 0 && res.starts_with( b"+PONG" ) {
                    let cmds = vec![
                        CMD_REPLCONF.to_string(),
                        "listening-port".to_string(),
                        state.port.to_string(),
                    ];

                    stream.write_all( encode_resp_arr( cmds )?.as_bytes() )?;
                    stream.flush()?;
                }

                if counter == 1 && res.starts_with( b"+OK" ) {
                    let cmds = vec![
                        CMD_REPLCONF.to_string(),
                        "capa".to_string(),
                        "psync2".to_string() 
                    ];

                    stream.write_all( encode_resp_arr( cmds )?.as_bytes() )?;
                    stream.flush()?;
                }


                if counter == 2 && res.starts_with( b"+OK" ) {
                    let cmds = vec![
                        CMD_PSYNC.to_string(),
                        "?".to_string(),
                        "-1".to_string(),
                    ];

                    stream.write_all( encode_resp_arr( cmds )?.as_bytes() )?;
                    stream.flush()?;
                }

                counter += 1;
            }

            Err( _ ) => todo!(),
        }
    }

    Ok( () )
}


impl AppState {
    pub fn from_args( args: &Vec<String> ) -> Result<AppState, Box<dyn std::error::Error>> {
        let mut state = AppState::default();

        for ( i, arg ) in args.iter().enumerate() {
            match arg.as_str() {
                ARG_PORT => {
                    handle_arg_port( &mut state, &args[ i + 1 ] )?;
                }

                ARG_REPLICAOF => {
                    handle_arg_replicaof( &mut state, &args[ i + 1 ] )?;
                }

                a => {
                    println!( "unknown arg: '{}'. Is it a value?", a );
                }
            }
        }

        Ok( state )
    }
}
