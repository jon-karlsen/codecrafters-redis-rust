use core::str;
use std::{io::{Read, Write}, net::TcpStream, slice::Iter, sync::{Arc, Mutex}, time::{Duration, Instant}};
use base64::prelude::*;
use crate::infra::app_state::{AppState, RedisStateValue, ServerRole};
use super::{constants::*, encode::{encode_bulk_string, encode_rdb_file, encode_resp_arr, encode_resp_str, encode_simple_str, RESP_STRING_START}};


fn handle_cmd_ping( stream: &mut TcpStream ) -> Result<(), Box<dyn std::error::Error>> {
    const PONG : &[ u8 ] = b"+PONG\r\n";

    stream.write_all( PONG )?;
    stream.flush()?;

    Ok( () )
}


fn handle_cmd_echo( stream: &mut TcpStream, args_it: &mut Iter<String> ) -> Result<(), Box<dyn std::error::Error>> {
    let arg = args_it.next().ok_or( "missing argument" )?; let ser = encode_simple_str( &arg )?;

    stream.write_all( ser.as_bytes() )?;
    stream.flush()?;

    Ok( () )
}


fn handle_cmd_replconfig( stream: &mut TcpStream, _state: &Arc<Mutex<AppState>> ) -> Result<(), Box<dyn std::error::Error>> {
    stream.write_all( b"+OK\r\n" )?;
    stream.flush()?;

    Ok( () )
}


fn handle_cmd_set( stream : &mut TcpStream,
                   args_it: &mut Iter<String>,
                   state  : &Arc<Mutex<AppState>> ) -> Result<(), Box<dyn std::error::Error>> {
    let mut px  = None;
    let     key = args_it.next().ok_or( "missing key" )?;
    let     val = args_it.next().ok_or( "missing val" )?;

    if let Some( _ ) = args_it.next() {
        let now = Instant::now();
        let dur = i32::from_str_radix( args_it.next().ok_or( "missing expiry" )?, 10 )?;
        let exp = now + Duration::from_millis( dur as u64 );

        px = Some( exp );
    }

    let val = RedisStateValue {
        expiry: px,
        value:  val.to_string(),
    };

    let mut state = state.lock().unwrap();

    state.redis_state.insert( key.to_string(), val.clone() );

    for mut s in &state.slave_connections {
        let enc = encode_resp_arr( vec![ CMD_SET, key, &val.value ] )?;

        s.write_all( enc.as_bytes() )?;
        s.flush()?;
    }

    stream.write_all( b"+OK\r\n" )?;
    stream.flush()?;

    Ok( () )
}


fn handle_cmd_get( stream : &mut TcpStream,
                   args_it: &mut Iter<String>,
                   state  : &Arc<Mutex<AppState>> ) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = state.lock().unwrap();

    let key = args_it.next().ok_or( "missing key" )?;
    let val = state.redis_state.get( key ).ok_or( "key not found" )?;

    if val.expiry.is_some() && val.expiry.unwrap() < Instant::now() {
        state.redis_state.remove( key );

        stream.write_all( b"$-1\r\n" )?;
        stream.flush()?;

        return Ok( () )
    }

    let ser = encode_simple_str( &val.value )?;

    stream.write_all( ser.as_bytes() )?;
    stream.flush()?;

    Ok( () )
}


fn handle_cmd_info( stream : &mut TcpStream,
                    args_it: &mut Iter<String>,
                    state  : &Arc<Mutex<AppState>> ) -> Result<(), Box<dyn std::error::Error>> {
    let section = args_it.next().ok_or( "missing section" )?;

    match section.as_str() {
        "replication" => {
            let     state  = state.lock().unwrap();
            let mut output = vec![];

            let role = match state.replication_info.role {
                ServerRole::Master => "master",
                ServerRole::Slave  => "slave",
            };

            let role               = format!( "role:{}", role );
            let master_replid      = format!( "master_replid:{}", state.master_replid.clone() );
            let master_repl_offset = format!( "master_repl_offset:{}", state.master_repl_offset );

            output.push( role.as_str() );

            if ! String::is_empty( &master_replid) {
                output.push( master_replid.as_str() );
            }

            output.push( master_repl_offset.as_str() );

            stream.write_all( encode_bulk_string( output )?.as_bytes() )?;

            stream.flush()?;
        }

        _ => {
            stream.write_all( b"+UNSUPPORTED\r\n" )?;
            stream.flush()?;
        }
    }

    Ok( () )
}


fn parse_args( buffer    : &mut [ u8 ],
               bytes_read: usize ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    println!( "parse_args" );

    let mut args: Vec<String> = vec![];

    for ( i, ch ) in buffer[ ..bytes_read ].iter().enumerate() {
        if *ch == RESP_STRING_START {
            let mut digits = 0;

            while buffer[ i + 1 + digits ] != b'\r' {
                digits += 1;
            }

            let len    = usize::from_str_radix( &str::from_utf8( &buffer[ i + 1..i + 1 + digits ] )?, 10 )?;
            let offset = if len > 9 { 5 } else { 4 };

            args.push( str::from_utf8( &buffer[ i + offset..i + offset + len ] )?.to_string() );
        }
    }

    Ok( args )
}


fn handle_cmd_psync( stream: &mut TcpStream,
                     state: &Arc<Mutex<AppState>> ) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = state.lock().unwrap();

    let reply = format!( "FULLRESYNC {} 0", state.master_replid.clone() );

    stream.write_all( encode_resp_str( &reply )?.as_bytes() )?;
    stream.flush()?;

    let rdb_empty_b64 = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";
    let rdb_bytes     = BASE64_STANDARD.decode( rdb_empty_b64 )?;
    let encoded       = encode_rdb_file( &rdb_bytes )?;

    stream.write_all( &encoded )?;
    stream.flush()?;

    state.slave_connections.push( stream.try_clone().unwrap() );

    Ok( () )
}


pub fn handle_connection( stream: &mut TcpStream, state: &Arc<Mutex<AppState>> ) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [ 0; 1024 ];

    loop {
        match stream.read( &mut buffer ) {
            Ok( 0 ) => {
                break;
            }

            Ok( bytes_read ) => {
                let args = parse_args( &mut buffer, bytes_read )?;

                println!( "args (cmd): {:?}", args );

                let mut args_it     = args.iter();
                let mut stream_copy = stream.try_clone().unwrap();

                match args_it.next() {
                    Some( cmd ) => {
                        match cmd.as_str() {
                            CMD_PING     => { let _ = handle_cmd_ping( &mut stream_copy ); }
                            CMD_ECHO     => { let _ = handle_cmd_echo( &mut stream_copy, &mut args_it ); }
                            CMD_SET      => { let _ = handle_cmd_set( &mut stream_copy, &mut args_it, &state ); }
                            CMD_GET      => { let _ = handle_cmd_get( &mut stream_copy, &mut args_it, &state ); }
                            CMD_INFO     => { let _ = handle_cmd_info( &mut stream_copy, &mut args_it, &state ); }
                            CMD_REPLCONF => { let _ = handle_cmd_replconfig( &mut stream_copy, &state ); }
                            CMD_PSYNC    => { let _ = handle_cmd_psync( &mut stream_copy, &state ); }

                            _ => {
                                stream.write_all( b"+OK\r\n" )?;
                                stream.flush()?;
                            }
                        }
                    }

                    None => {
                        stream.write_all( b"+NULL\r\n" )?;
                        stream.flush()?;
                    }
                }
            }

            Err( e ) => {
                return Err( Box::new( e ) );
            }
        }
    }

    Ok( () )
}


#[cfg( test )]
mod tests {
    use base64::prelude::*;

    use crate::resp::encode::encode_rdb_file;


    #[test]
    fn test_decode_base64() -> Result<(), Box<dyn std::error::Error>> {
        let rdb_empty_b64 = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";

        let rdb_bytes     = BASE64_STANDARD.decode( rdb_empty_b64 )?;
        println!( "rdb_bytes: {:?}", rdb_bytes );

        let encoded       = encode_rdb_file( &rdb_bytes )?;
        println!( "encoded: {:?}", encoded );

        Ok( () )
    }
}
