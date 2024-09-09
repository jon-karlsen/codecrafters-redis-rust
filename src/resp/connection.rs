use core::str;
use std::{collections::HashMap, io::{Read, Write}, net::TcpStream, string::FromUtf8Error};


const RESP_ARRAY_START : u8 = b'*';
const RESP_STRING_START: u8 = b'$';

const PONG   : &[ u8 ] = b"+PONG\r\n";


#[derive( Debug, PartialEq )]
pub enum RespError {
    BufferReadErr( String ),
    ParseErr,
    FmtError( String ),
}


#[derive( Default )]
pub struct Resp {
    pub len : u32,
    pub cmd : String,
    pub args: Option<Vec<String>>
}


enum Mode {
    Initial,
    ReadLen,
    ReadCmd,
    ReadArgs
}


impl Resp {
    pub fn handle_connection( stream: &mut TcpStream ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer                         = [ 0; 1024 ];
        let mut state: HashMap<String, String> = HashMap::new();

        loop {
            match stream.read( &mut buffer ) {
                Ok( 0 ) => {
                    break;
                }

                Ok( bytes_read ) => {
                    let mut args: Vec<String> = vec![];

                    for ( i, ch ) in buffer[ ..bytes_read ].iter().enumerate() {
                        if *ch == RESP_STRING_START {
                            let len = char::to_digit( buffer[ i + 1 ] as char, 10 ).unwrap() as usize;
                            args.push( str::from_utf8( &buffer[ i + 4..i + 4 + len ] )?.to_string() );
                        }
                    }

                    println!( "args: {:?}", args );

                    let mut args_it = args.iter();

                    match args_it.next() {
                        Some( cmd ) if cmd == "PING" => {
                            stream.write_all( PONG )?;
                            stream.flush()?;
                        }

                        Some( cmd ) if cmd == "ECHO" => {
                            let arg = args_it.next().ok_or( "missing argument" )?;
                            let ser = serialize_resp_str( &arg )?;

                            stream.write_all( ser.as_bytes() )?;
                            stream.flush()?;
                        }

                        Some( cmd ) if cmd == "SET" => {
                            let key = args_it.next().ok_or( "missing key" )?;
                            let val = args_it.next().ok_or( "missing val" )?;

                            state.insert( key.to_string(), val.to_string() );

                            stream.write_all( b"+OK\r\n" )?;
                            stream.flush()?;
                        }

                        Some( cmd ) if cmd == "GET" => {
                            let key = args_it.next().ok_or( "missing key" )?;
                            let val = state.get( key ).ok_or( "key not found" )?;
                            let ser = serialize_resp_str( &val )?;

                            stream.write_all( ser.as_bytes() )?;
                            stream.flush()?;
                        }

                        Some( _ ) => {
                            stream.write_all( b"+OK\r\n" )?;
                            stream.flush()?;
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
}


pub fn serialize_resp_str( input: &str ) -> Result<String, FromUtf8Error>  {
    let mut result = vec![
        RESP_STRING_START,
        char::from_digit( input.len() as u32, 10 ).unwrap() as u8
    ];

    result.push( b'\r' );
    result.push( b'\n' );

    result.extend( input.as_bytes() );

    result.push( b'\r' );
    result.push( b'\n' );

    String::from_utf8( result )
}
