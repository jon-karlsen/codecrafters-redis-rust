pub const RESP_ARRAY_START        : u8 = b'*';
pub const RESP_STRING_START       : u8 = b'$';
pub const RESP_SIMPLE_STRING_START: u8 = b'+';


pub fn encode_simple_str( input: &str ) -> anyhow::Result<String> {
    let mut result = vec![
        RESP_SIMPLE_STRING_START
    ];

    result.extend( input.as_bytes() );

    result.push( b'\r' );
    result.push( b'\n' );

    Ok( String::from_utf8( result )? )
}


pub fn encode_resp_str( input: &str ) -> anyhow::Result<String>  {
    let mut result = vec![
        RESP_STRING_START,
    ];

    for b in input.len().to_string().as_bytes() {
        result.push( *b );
    }

    result.push( b'\r' );
    result.push( b'\n' );

    result.extend( input.as_bytes() );

    result.push( b'\r' );
    result.push( b'\n' );

    Ok( String::from_utf8( result )? )
}


pub fn encode_bulk_string( parts: Vec<String> ) -> anyhow::Result<String> {
    let mut wrapper = String::from( "$" );
    let mut content = String::new();

    for ( i, part ) in parts.iter().enumerate() {
        if i > 0 {
            content.push_str( "," );
        }

        content.push_str( part );
    }

    wrapper.push_str( &content.len().to_string() );
    wrapper.push_str( "\r\n" );
    wrapper.push_str( &content );
    wrapper.push_str( "\r\n" );

    Ok( wrapper )
}


pub fn encode_rdb_file( input: &Vec<u8> ) -> anyhow::Result<Vec<u8>> {
    let mut result = vec![
        RESP_STRING_START,
    ];

    for b in input.len().to_string().as_bytes() {
        result.push( *b );
    }

    result.push( b'\r' );
    result.push( b'\n' );

    result.extend( input );

    Ok( result )
}


pub fn encode_resp_arr( parts: Vec<String> ) -> anyhow::Result<String> {
    let mut wrapper = String::from( "*" );
    let mut content = String::new();

    for part in parts.iter() {
        let s = encode_bulk_string( vec![ part.to_owned() ] )?;

        content.push_str( &s );
    }

    wrapper.push_str( &parts.len().to_string() );
    wrapper.push_str( "\r\n" );

    wrapper.push_str( &content );

    Ok( wrapper )
}
