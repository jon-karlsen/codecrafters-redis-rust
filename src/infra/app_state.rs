use std::{collections::HashMap, net::TcpStream, time::Instant};


const DEFAULT_PORT      : i32  = 6379;
const DEMO_MASTER_REPLID: &str = "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb";


pub enum ServerRole {
    Master = 0,
    Slave  = 1,
}


pub struct ReplicationInfo {
    pub role: ServerRole
}


impl Default for ReplicationInfo {
    fn default() -> Self {
        Self {
            role: ServerRole::Master
        }
    }
}


pub struct RedisStateValue {
    pub expiry: Option<Instant>,
    pub value:  String,
}


impl Clone for RedisStateValue {
    fn clone(&self) -> Self {
        Self {
            expiry: self.expiry.clone(),
            value:  self.value.clone(),
        }
    }
}


pub struct AppState {
    pub     _master_host      : String,
    pub     _master_port      : i32,
    pub     master_replid     : String,
    pub     master_repl_offset: usize,
    pub     port              : i32,
    pub     replication_info  : ReplicationInfo,
    pub     redis_state       : HashMap<String, RedisStateValue>,
    pub     slave_connections : Vec<TcpStream>,
}


impl Default for AppState {
    fn default() -> Self {
        Self {
            _master_host       : "localhost".to_string(),
            _master_port       : 6379,
            master_replid     : DEMO_MASTER_REPLID.to_string(),
            master_repl_offset: 0,
            port              : DEFAULT_PORT,
            replication_info  : ReplicationInfo::default(),
            redis_state       : HashMap::new(),
            slave_connections : vec![],
        }
    }
}
