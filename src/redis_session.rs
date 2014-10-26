extern crate redis;
extern crate session;
extern crate collections;

use std::sync::Arc;
use redis::Commands;
use redis::{ToRedisArgs, FromRedisValue};
use redis::RedisResult;
use self::session::SessionStore;
use self::collections::hash::Hash;

pub struct RedisSessionStore<K, V> {
 client: Arc<redis::Client>,
}

impl<K: Clone + Send, V: Send> Clone for RedisSessionStore<K, V> {
  fn clone(&self) -> RedisSessionStore<K, V> {
    RedisSessionStore {
      client: self.client.clone()
    }
  }
}

impl<K: Hash + Eq + Send + Sync, V: Send + Sync> RedisSessionStore<K, V> {
  pub fn new(c: redis::Client) -> RedisSessionStore<K, V> {
    RedisSessionStore {
      client: Arc::new(c)
    }
  }
}

impl<K: Hash + Eq + Send + Sync + Clone + ToRedisArgs, V: Send + Sync + Clone + ToRedisArgs + FromRedisValue> SessionStore<K, V> for RedisSessionStore<K, V> {

  fn insert(&self, key: &K, val: V) {
    let con = self.client.get_connection();
    let _ : () = con.set(key.clone(), val).unwrap();
  }

  fn find(&self, key: &K) -> Option<V> {
    let result: RedisResult<V> = self.client.get(key.clone());
    match result {
      Ok(v) => Some(v.clone()),
      _     => None
    }
  }

  fn swap(&self, key: &K, value: V) -> Option<V> {
    let con = self.client.get_connection();
    let result: RedisResult<V> = redis::cmd("GETSET").arg(key.clone()).arg(value).query(&con);
    match result {
      Ok(v) => Some(v.clone()),
      _     => None
    }
  }

  fn upsert(&self, key: &K, value: V, mutator: |&mut V|) -> V {
    let result: RedisResult<V> = self.client.get(key.clone());
    match result {
      Ok(v) => {
        let old_v = &mut v.clone();
        mutator(old_v);
        self.insert(key, old_v.clone());
        return old_v.clone();
      },
      _     => {
        self.insert(key, value.clone());
        return value;
      }
    }
  }

  fn remove(&self, key: &K) -> bool {
    let result: RedisResult<V> = self.client.del(key.clone());
    return true;
  }
}
