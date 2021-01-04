[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE) [![](http://meritbadge.herokuapp.com/r2d2_redis_cluster)](https://crates.io/crates/r2d2_redis_cluster)

Redis cluster support for the `r2d2` connection pool.

Documentation is available at [here](https://docs.rs/r2d2_redis_cluster/0.1.6/r2d2_redis_cluster/).

# Example

```rust,no_run
extern crate r2d2_redis_cluster;

use std::thread;

use r2d2_redis_cluster::{r2d2::Pool, Commands, RedisClusterConnectionManager};

fn main() {
    let redis_uri = vec!["redis://127.0.0.1:6379", "redis://127.0.0.1:6378", "redis://127.0.0.1:6377"];
    let manager = RedisClusterConnectionManager::new(redis_uri).unwrap();
    let pool = Pool::builder()
        .build(manager)
        .unwrap();

    let mut handles = Vec::new();

    for _ in 0..10 {
        let pool = pool.clone();
        handles.push(thread::spawn(move || {
            let connection = pool.get().unwrap();
            let n: u64 = connection.incr("test", 1).unwrap();
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let mut connection = pool.get().unwrap();
    let res: u64 = connection.get("test").unwrap();

    assert_eq!(res, 10);
}
```
