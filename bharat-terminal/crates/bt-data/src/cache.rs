// crates/bt-data/src/cache.rs
// Author: Sourish Dey

use bt_core::{BtError, Candle, Result};
use chrono::{Duration, Utc};
use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Mutex;
use tracing::instrument;

fn to_bt_err(e: rusqlite::Error) -> BtError {
    BtError::Database(e.to_string())
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS ohlcv (
    symbol TEXT NOT NULL,
    interval TEXT NOT NULL,
    ts INTEGER NOT NULL,
    o REAL NOT NULL,
    h REAL NOT NULL,
    l REAL NOT NULL,
    c REAL NOT NULL,
    v REAL NOT NULL,
    fetched_at INTEGER NOT NULL,
    PRIMARY KEY (symbol, interval, ts)
);

CREATE INDEX IF NOT EXISTS idx_ohlcv_symbol_interval ON ohlcv(symbol, interval);
CREATE INDEX IF NOT EXISTS idx_ohlcv_fetched_at ON ohlcv(fetched_at);
";

/// SQLite-backed cache for OHLCV data.
/// TTL: 5 minutes for intraday intervals, 24 hours for daily+ intervals.
pub struct Cache {
    conn: Mutex<Connection>,
}

impl Cache {
    /// Open or create cache at `path`.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path).map_err(to_bt_err)?;
        conn.execute_batch(SCHEMA).map_err(to_bt_err)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Get cached OHLCV data for symbol and interval.
    /// Returns None if cache miss or data expired.
    #[instrument(skip(self))]
    pub fn get_ohlcv(&self, symbol: &str, interval: &str) -> Result<Option<Vec<Candle>>> {
        let ttl_secs = self.ttl_for_interval(interval);
        let cutoff = (Utc::now() - Duration::seconds(ttl_secs)).timestamp();

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT ts, o, h, l, c, v FROM ohlcv
                 WHERE symbol = ?1 AND interval = ?2 AND fetched_at > ?3
                 ORDER BY ts",
            )
            .map_err(to_bt_err)?;

        let rows = stmt
            .query_map(params![symbol, interval, cutoff], |row| {
                Ok(Candle {
                    t: row.get::<_, i64>(0)? as f64,
                    open: row.get(1)?,
                    high: row.get(2)?,
                    low: row.get(3)?,
                    close: row.get(4)?,
                    volume: row.get(5)?,
                })
            })
            .map_err(to_bt_err)?;

        let mut candles = Vec::new();
        for row in rows {
            candles.push(row.map_err(to_bt_err)?);
        }

        if candles.is_empty() {
            Ok(None)
        } else {
            Ok(Some(candles))
        }
    }

    /// Store OHLCV data in cache.
    #[instrument(skip(self, candles))]
    pub fn put_ohlcv(&self, symbol: &str, interval: &str, candles: &[Candle]) -> Result<()> {
        if candles.is_empty() {
            return Ok(());
        }

        let now = Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction().map_err(to_bt_err)?;

        {
            let mut stmt = tx
                .prepare(
                    "INSERT OR REPLACE INTO ohlcv (symbol, interval, ts, o, h, l, c, v, fetched_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                )
                .map_err(to_bt_err)?;

            for c in candles {
                stmt.execute(params![
                    symbol,
                    interval,
                    c.t as i64,
                    c.open,
                    c.high,
                    c.low,
                    c.close,
                    c.volume,
                    now
                ])
                .map_err(to_bt_err)?;
            }
        }

        tx.commit().map_err(to_bt_err)?;
        Ok(())
    }

    /// Clear expired entries (older than max TTL).
    #[instrument(skip(self))]
    pub fn cleanup(&self) -> Result<usize> {
        let max_ttl = 86400 * 7;
        let cutoff = (Utc::now() - Duration::seconds(max_ttl)).timestamp();

        let conn = self.conn.lock().unwrap();
        let deleted = conn
            .execute("DELETE FROM ohlcv WHERE fetched_at < ?1", params![cutoff])
            .map_err(to_bt_err)?;

        Ok(deleted)
    }

    /// Get cache stats.
    #[instrument(skip(self))]
    pub fn stats(&self) -> Result<CacheStats> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM ohlcv", [], |row| row.get(0))
            .map_err(to_bt_err)?;
        let size: i64 = conn
            .query_row(
                "SELECT SUM(length(symbol) + length(interval) + 56) FROM ohlcv",
                [],
                |row| row.get(0),
            )
            .map_err(to_bt_err)
            .unwrap_or(0);

        Ok(CacheStats {
            entries: count as usize,
            approx_size_bytes: size as usize,
        })
    }

    fn ttl_for_interval(&self, interval: &str) -> i64 {
        match interval {
            "1m" | "5m" | "15m" | "30m" | "1h" => 300,
            "1d" | "1wk" | "1mo" => 86400,
            _ => 300,
        }
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub approx_size_bytes: usize,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new("./data/cache.db").expect("Failed to create default cache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_cache.db");
        let cache = Cache::new(&path).unwrap();

        let candles = vec![
            Candle::new(1_000_000.0, 100.0, 105.0, 99.0, 103.0, 1000.0),
            Candle::new(1_000_001.0, 103.0, 107.0, 102.0, 105.0, 1500.0),
        ];

        cache.put_ohlcv("TEST", "1d", &candles).unwrap();

        let retrieved = cache.get_ohlcv("TEST", "1d").unwrap().unwrap();
        assert_eq!(retrieved.len(), 2);
        assert!((retrieved[0].open - 100.0).abs() < f64::EPSILON);
        assert!((retrieved[1].close - 105.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_miss() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_cache.db");
        let cache = Cache::new(&path).unwrap();

        let result = cache.get_ohlcv("NONEXISTENT", "1d").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_expiry() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_cache.db");
        let cache = Cache::new(&path).unwrap();

        let candles = vec![Candle::new(1_000_000.0, 100.0, 105.0, 99.0, 103.0, 1000.0)];

        let old_time = (Utc::now() - Duration::seconds(400)).timestamp();
        cache
            .conn
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO ohlcv VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params!["TEST", "1m", 1_000_000, 100.0, 105.0, 99.0, 103.0, 1000.0, old_time],
            )
            .unwrap();

        let result = cache.get_ohlcv("TEST", "1m").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_daily_cache_not_expired() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_cache.db");
        let cache = Cache::new(&path).unwrap();

        let candles = vec![Candle::new(1_000_000.0, 100.0, 105.0, 99.0, 103.0, 1000.0)];

        let old_time = (Utc::now() - Duration::seconds(12 * 3600)).timestamp();
        cache
            .conn
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO ohlcv VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params!["TEST", "1d", 1_000_000, 100.0, 105.0, 99.0, 103.0, 1000.0, old_time],
            )
            .unwrap();

        let result = cache.get_ohlcv("TEST", "1d").unwrap();
        assert!(result.is_some());
    }
}