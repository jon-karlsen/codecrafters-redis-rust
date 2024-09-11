use std::{collections::HashMap, time::Instant};

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
    pub redis_state     : HashMap<String, RedisStateValue>,
    pub port            : i32,
}


impl Default for AppState {
    fn default() -> Self {
        Self {
            replication_info: ReplicationInfo::default(),
            redis_state     : HashMap::new(),
            port            : 6379,
        }
    }
}
