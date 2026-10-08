use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use axum::http::HeaderMap;
use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlRow};
use tokio::sync::{Mutex, RwLock};

use crate::error::{ApiError, ApiResult};

pub type BoxFut<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Build the session connection pool: single connection, never recycled, equivalent to phpMyAdmin's single persistent connection
/// (USE/session variables and similar state stay valid for the connection's lifetime).
pub async fn session_pool(opts: MySqlConnectOptions) -> sqlx::Result<MySqlPool> {
    use sqlx::mysql::MySqlPoolOptions;
    MySqlPoolOptions::new()
        .max_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(opts)
        .await
}

fn is_disconnected(e: &sqlx::Error) -> bool {
    if matches!(
        e,
        sqlx::Error::Io(_) | sqlx::Error::Protocol(_) | sqlx::Error::WorkerCrashed
    ) {
        return true;
    }
    match e.as_database_error() {
        Some(db) => matches!(db.code().as_deref(), Some("2006") | Some("2013")),
        None => false,
    }
}

// ============ Commands and outputs ============

    /// Command executable within a session (carries all input to avoid closure/lifetime issues)
#[derive(Debug, Clone)]
pub enum Cmd {
    /// Execute arbitrary statements via text protocol (USE/DDL/DML...), returns (rows affected, last insert ID)
    Exec(String),
    /// Fetch rows via text protocol
    Rows(String),
    /// Execute multiple statements inside a transaction
    Tx(Vec<String>),
    /// DESCRIBE to get result column names
    Describe(String),
}

pub enum CmdOut {
    Exec((u64, u64)),
    Rows(Vec<MySqlRow>),
    Tx(u64),
    Describe(Vec<String>),
}

/// Single-attempt result classification: disconnected (retryable once) vs ordinary failure
enum CmdErr {
    Disconnected,
    Failed(ApiError),
}

/// Each session holds a single persistent connection to MySQL (max=1 pool, matching phpMyAdmin),
/// so `USE`, session variables, and similar state take effect for subsequent statements.
///
/// Execution shape note: all queries are submitted through `&MySqlPool`. Do not pass
/// `&mut MySqlConnection` as a parameter to an async helper and await it in an outer scope,
/// and do not put a concrete future holding sqlx connection borrow chains directly into a
/// `dyn + Send` box or spawn it — that is the sqlx-documented "implementation of Executor
/// is not general enough" (rustc higher-ranked trait solving limitation) trigger shape.
pub struct Session {
    pub host: String,
    pub display_name: String,
    pool: MySqlPool,
    /// Command-level mutex: guarantees single-connection serial semantics and explicit-transaction atomicity
    lock: Mutex<()>,
    pub last_used: Mutex<std::time::Instant>,
}

/// Single-attempt executor: cmd consumed by value, all queries go through &pool (borrows stay frame-local).
async fn run_cmd_once(this: Arc<Session>, cmd: Cmd) -> Result<CmdOut, CmdErr> {
    use sqlx::{Column, Executor};

    // Serialize the whole command (including transactions); other commands queue while the lock is held
    let _guard = this.lock.lock().await;
    let pool = this.pool.clone();

    let result: sqlx::Result<CmdOut> = match cmd {
        Cmd::Exec(sql) => sqlx::raw_sql(sql.as_str())
            .execute(&pool)
            .await
            .map(|r| CmdOut::Exec((r.rows_affected(), r.last_insert_id()))),
        Cmd::Rows(sql) => sqlx::raw_sql(sql.as_str())
            .fetch_all(&pool)
            .await
            .map(CmdOut::Rows),
        Cmd::Tx(stmts) => {
            // Explicit transaction: single connection + command-level mutex ⇒ BEGIN/.../COMMIT cannot be interleaved
            match sqlx::raw_sql("BEGIN").execute(&pool).await {
                Ok(_) => {
                    let mut affected: u64 = 0;
                    let mut err: Option<sqlx::Error> = None;
                    for s in &stmts {
                        match sqlx::raw_sql(s.as_str()).execute(&pool).await {
                            Ok(r) => affected += r.rows_affected(),
                            Err(e) => {
                                err = Some(e);
                                break;
                            }
                        }
                    }
                    match err {
                        Some(e) => {
                            let _ = sqlx::raw_sql("ROLLBACK").execute(&pool).await;
                            Err(e)
                        }
                        None => match sqlx::raw_sql("COMMIT").execute(&pool).await {
                            Ok(_) => Ok(CmdOut::Tx(affected)),
                            Err(e) => {
                                let _ = sqlx::raw_sql("ROLLBACK").execute(&pool).await;
                                Err(e)
                            }
                        },
                    }
                }
                Err(e) => Err(e),
            }
        }
        Cmd::Describe(sql) => match (&pool).describe(sql.as_str()).await {
            Ok(d) => Ok(CmdOut::Describe(
                d.columns.iter().map(|c| c.name().to_string()).collect(),
            )),
            Err(e) => Err(e),
        },
    };
    drop(_guard);

    match result {
        Ok(v) => {
            *this.last_used.lock().await = std::time::Instant::now();
            Ok(v)
        }
        Err(e) => Err(if is_disconnected(&e) {
            CmdErr::Disconnected
        } else {
            CmdErr::Failed(e.into())
        }),
    }
}

fn cmd_err_into(e: CmdErr) -> ApiError {
    match e {
        CmdErr::Disconnected => ApiError::connection_lost("数据库连接已失效"),
        CmdErr::Failed(e) => e,
    }
}

/// Command executor (the pool auto-reconnects and retries once on disconnect). All parameters are owned.
async fn run_cmd_inner(this: Arc<Session>, cmd: Cmd) -> ApiResult<CmdOut> {
    match run_cmd_once(this.clone(), cmd.clone()).await {
        Err(CmdErr::Disconnected) => run_cmd_once(this, cmd).await.map_err(cmd_err_into),
        other => other.map_err(cmd_err_into),
    }
}

impl Session {
    pub fn new(host: String, display_name: String, pool: MySqlPool) -> Arc<Self> {
        Arc::new(Self {
            host,
            display_name,
            pool,
            lock: Mutex::new(()),
            last_used: Mutex::new(std::time::Instant::now()),
        })
    }

    /// Execute a command on the session connection; auto-reconnect and retry once on disconnect.
    pub fn run_cmd(self: &Arc<Self>, cmd: Cmd) -> BoxFut<ApiResult<CmdOut>> {
        let this = Arc::clone(self);
        Box::pin(async move { run_cmd_inner(this, cmd).await })
    }

    // ============ High-level convenience methods ============

    pub fn raw_exec(self: &Arc<Self>, sql: impl Into<String>) -> BoxFut<ApiResult<(u64, u64)>> {
        let this = Arc::clone(self);
        let cmd = Cmd::Exec(sql.into());
        Box::pin(async move {
            match this.run_cmd(cmd).await? {
                CmdOut::Exec(v) => Ok(v),
                _ => Err(ApiError::internal("内部错误：命令输出类型不匹配")),
            }
        })
    }

    pub fn raw_exec_ok(self: &Arc<Self>, sql: impl Into<String>) -> BoxFut<ApiResult<()>> {
        let this = Arc::clone(self);
        let cmd = Cmd::Exec(sql.into());
        Box::pin(async move {
            this.run_cmd(cmd).await?;
            Ok(())
        })
    }

    pub fn raw_rows(self: &Arc<Self>, sql: impl Into<String>) -> BoxFut<ApiResult<Vec<MySqlRow>>> {
        let this = Arc::clone(self);
        let cmd = Cmd::Rows(sql.into());
        Box::pin(async move {
            match this.run_cmd(cmd).await? {
                CmdOut::Rows(v) => Ok(v),
                _ => Err(ApiError::internal("内部错误：命令输出类型不匹配")),
            }
        })
    }

    pub fn exec_tx(self: &Arc<Self>, stmts: Vec<String>) -> BoxFut<ApiResult<u64>> {
        let this = Arc::clone(self);
        let cmd = Cmd::Tx(stmts);
        Box::pin(async move {
            match this.run_cmd(cmd).await? {
                CmdOut::Tx(v) => Ok(v),
                _ => Err(ApiError::internal("内部错误：命令输出类型不匹配")),
            }
        })
    }

    pub fn describe_columns(self: &Arc<Self>, sql: impl Into<String>) -> BoxFut<ApiResult<Vec<String>>> {
        let this = Arc::clone(self);
        let cmd = Cmd::Describe(sql.into());
        Box::pin(async move {
            match this.run_cmd(cmd).await? {
                CmdOut::Describe(v) => Ok(v),
                _ => Err(ApiError::internal("内部错误：命令输出类型不匹配")),
            }
        })
    }

    pub fn close(self: &Arc<Self>) -> BoxFut<()> {
        Box::pin(close_inner(Arc::clone(self)))
    }
}

async fn close_inner(this: Arc<Session>) {
    this.pool.close().await;
}

pub struct AppState {
    sessions: RwLock<HashMap<String, Arc<Session>>>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            sessions: RwLock::new(HashMap::new()),
        })
    }

    pub async fn insert(&self, sess: Arc<Session>) -> String {
        let token = uuid::Uuid::new_v4().simple().to_string();
        self.sessions.write().await.insert(token.clone(), sess);
        token
    }

    pub async fn get(&self, token: &str) -> Option<Arc<Session>> {
        self.sessions.read().await.get(token).cloned()
    }

    pub async fn remove(&self, token: &str) {
        if let Some(s) = self.sessions.write().await.remove(token) {
            s.close().await;
        }
    }

    pub async fn auth(&self, headers: &HeaderMap) -> ApiResult<Arc<Session>> {
        let token = headers
            .get("x-auth-token")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(ApiError::unauthorized)?;
        self.get(token).await.ok_or_else(ApiError::unauthorized)
    }

    /// Clean up sessions idle for over 24 hours
    pub async fn cleanup(&self) {
        const IDLE_LIMIT: Duration = Duration::from_secs(24 * 3600);
        let mut stale = Vec::new();
        {
            let map = self.sessions.read().await;
            for (k, v) in map.iter() {
                if v.last_used.lock().await.elapsed() > IDLE_LIMIT {
                    stale.push(k.clone());
                }
            }
        }
        for k in stale {
            self.remove(&k).await;
        }
    }
}
