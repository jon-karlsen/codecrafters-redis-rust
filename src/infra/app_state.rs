use std::{collections::HashMap, time::Instant};


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


pub struct AppState {
    pub replication_info: ReplicationInfo,
    pub redis_state       : HashMap<String, RedisStateValue>,
    pub master_replid     : String,
    pub master_repl_offset: usize,
    pub port              : i32,
}


impl Default for AppState {
    fn default() -> Self {
        Self {
            replication_info: ReplicationInfo::default(),
            redis_state       : HashMap::new(),
            master_replid     : DEMO_MASTER_REPLID.to_string(),
            master_repl_offset: 0,
            port              : DEFAULT_PORT,
        }
    }
}
