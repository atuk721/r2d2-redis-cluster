//! Redis cluster support for the `r2d2` connection pool.
//!
//! # Example
//! ```rust,no_run
//! extern crate r2d2_redis_cluster;
//!
//! use std::thread;
//!
//! use r2d2_redis_cluster::{r2d2::Pool, Commands, RedisClusterConnectionManager};
//!
//! fn main() {
//!     let redis_uri = vec!["redis://127.0.0.1:6379", "redis://127.0.0.1:6378", "redis://127.0.0.1:6377"];
//!     let manager = RedisClusterConnectionManager::new(redis_uri).unwrap();
//!     let pool = Pool::builder()
//!         .build(manager)
//!         .unwrap();
//!
//!     let mut handles = Vec::new();
//!
//!     for _ in 0..10 {
//!         let pool = pool.clone();
//!         handles.push(thread::spawn(move || {
//!             let connection = pool.get().unwrap();
//!             let _: u64 = connection.incr("test", 1).unwrap();
//!         }));
//!     }
//!
//!     for h in handles {
//!         h.join().unwrap();
//!     }
//!
//!     let mut connection = pool.get().unwrap();
//!     let res: u64 = connection.get("test").unwrap();
//!
//!     assert_eq!(res, 10);
//! }
//! ```
pub extern crate r2d2;
pub extern crate redis_cluster_rs;

use r2d2::ManageConnection;
use redis_cluster_rs::{
    redis::{ConnectionInfo, ErrorKind, IntoConnectionInfo, RedisError},
    Builder, Connection,
};

pub use redis_cluster_rs::redis::{Commands, ConnectionLike, RedisResult};

/// An `r2d2::ConnectionManager` for `redis_cluster_rs::Client`.
#[derive(Debug)]
pub struct RedisClusterConnectionManager {
    nodes: Vec<ConnectionInfo>,
    readonly: bool,
    password: Option<String>,
}

impl RedisClusterConnectionManager {
    /// Create new `RedisClusterConnectionManager`.
    pub fn new<T: IntoConnectionInfo>(
        input_nodes: Vec<T>,
    ) -> RedisResult<RedisClusterConnectionManager> {
        let mut nodes = Vec::with_capacity(input_nodes.len());

        for node in input_nodes {
            nodes.push(node.into_connection_info()?)
        }

        Ok(RedisClusterConnectionManager {
            nodes,
            readonly: false,
            password: None,
        })
    }

    /// Create new `RedisClusterConnectionManager` with authentication.
    #[deprecated(note = "Please use new and password function")]
    pub fn new_with_auth<T: IntoConnectionInfo>(
        input_nodes: Vec<T>,
        password: String,
    ) -> RedisResult<RedisClusterConnectionManager> {
        let mut result = Self::new(input_nodes)?;
        result.set_password(password);
        Ok(result)
    }

    /// Set read only mode for new Connection.
    pub fn set_readonly(&mut self, readonly: bool) {
        self.readonly = readonly;
    }

    /// Set password for new Connection.
    pub fn set_password(&mut self, password: String) {
        self.password = Some(password);
    }
}

impl ManageConnection for RedisClusterConnectionManager {
    type Connection = Connection;
    type Error = RedisError;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let builder = Builder::new(self.nodes.clone()).readonly(self.readonly);

        let client = if let Some(password) = self.password.clone() {
            builder.password(password).open()?
        } else {
            builder.open()?
        };
        client.get_connection()
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        if conn.check_connection() {
            Ok(())
        } else {
            Err(RedisError::from((
                ErrorKind::IoError,
                "Connection check error.",
            )))
        }
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}
