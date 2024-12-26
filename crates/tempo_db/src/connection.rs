// This is copied from `tokio-rusqlite`, which can be found here:
// https://github.com/programatik29/tokio-rusqlite/tree/master
// Copied it in case I want to make any changes.

// This file is MIT licensed.

#![forbid(unsafe_code)]
#![warn(
    clippy::await_holding_lock,
    clippy::cargo_common_metadata,
    clippy::dbg_macro,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::inefficient_to_string,
    clippy::mem_forget,
    clippy::mutex_integer,
    clippy::needless_continue,
    clippy::todo,
    clippy::unimplemented,
    clippy::wildcard_imports,
    future_incompatible,
    missing_docs,
    missing_debug_implementations,
    unreachable_pub
)]

use crossbeam::channel::{Receiver, Sender};
use std::{
    fmt::{self, Debug, Display},
    path::Path,
    thread,
};
use tokio::sync::oneshot::{self};

use rusqlite::*;

const BUG_TEXT: &str = "bug in tokio-rusqlite, please report";

#[derive(Debug)]
/// Represents the errors specific for this library.
#[non_exhaustive]
pub enum ConnectionError {
    /// The connection to the SQLite has been closed and cannot be queried any more.
    ConnectionClosed,

    /// An error occured while closing the SQLite connection.
    /// This `Error` variant contains the [`Connection`], which can be used to retry the close operation
    /// and the underlying [`rusqlite::Error`] that made it impossile to close the database.
    Close((Connection, rusqlite::Error)),

    /// A `Rusqlite` error occured.
    Rusqlite(rusqlite::Error),

    /// An application-specific error occured.
    Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::ConnectionClosed => write!(f, "ConnectionClosed"),
            ConnectionError::Close((_, e)) => write!(f, "Close((Connection, \"{e}\"))"),
            ConnectionError::Rusqlite(e) => write!(f, "Rusqlite(\"{e}\")"),
            ConnectionError::Other(ref e) => write!(f, "Other(\"{e}\")"),
        }
    }
}

impl std::error::Error for ConnectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConnectionError::ConnectionClosed => None,
            ConnectionError::Close((_, e)) => Some(e),
            ConnectionError::Rusqlite(e) => Some(e),
            ConnectionError::Other(ref e) => Some(&**e),
        }
    }
}

impl From<rusqlite::Error> for ConnectionError {
    fn from(value: rusqlite::Error) -> Self {
        ConnectionError::Rusqlite(value)
    }
}

impl From<anyhow::Error> for ConnectionError {
    fn from(value: anyhow::Error) -> Self {
        ConnectionError::Other(value.into())
    }
}

/// The result returned on method calls in this crate.
type Result<T> = std::result::Result<T, ConnectionError>;

type CallFn = Box<dyn FnOnce(&mut rusqlite::Connection) + Send + 'static>;

enum Message {
    Execute(CallFn),
    Close(oneshot::Sender<std::result::Result<(), rusqlite::Error>>),
}

/// A handle to call functions in background thread.
#[derive(Clone)]
pub struct Connection {
    sender: Sender<Message>,
}

impl Connection {
    /// Open a new connection to a SQLite database.
    ///
    /// `Connection::open(path)` is equivalent to
    /// `Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE |
    /// OpenFlags::SQLITE_OPEN_CREATE)`.
    ///
    /// # Failure
    ///
    /// Will return `Err` if `path` cannot be converted to a C-compatible
    /// string or if the underlying SQLite open call fails.
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_owned();
        start(move || rusqlite::Connection::open(path))
            .await
            .map_err(ConnectionError::Rusqlite)
    }

    /// Open a new connection to an in-memory SQLite database.
    ///
    /// # Failure
    ///
    /// Will return `Err` if the underlying SQLite open call fails.
    pub async fn open_in_memory() -> Result<Self> {
        start(rusqlite::Connection::open_in_memory)
            .await
            .map_err(ConnectionError::Rusqlite)
    }

    /// Open a new connection to a SQLite database.
    ///
    /// [Database Connection](http://www.sqlite.org/c3ref/open.html) for a
    /// description of valid flag combinations.
    ///
    /// # Failure
    ///
    /// Will return `Err` if `path` cannot be converted to a C-compatible
    /// string or if the underlying SQLite open call fails.
    pub async fn open_with_flags<P: AsRef<Path>>(path: P, flags: OpenFlags) -> Result<Self> {
        let path = path.as_ref().to_owned();
        start(move || rusqlite::Connection::open_with_flags(path, flags))
            .await
            .map_err(ConnectionError::Rusqlite)
    }

    /// Open a new connection to a SQLite database using the specific flags
    /// and vfs name.
    ///
    /// [Database Connection](http://www.sqlite.org/c3ref/open.html) for a
    /// description of valid flag combinations.
    ///
    /// # Failure
    ///
    /// Will return `Err` if either `path` or `vfs` cannot be converted to a
    /// C-compatible string or if the underlying SQLite open call fails.
    pub async fn open_with_flags_and_vfs<P: AsRef<Path>>(
        path: P,
        flags: OpenFlags,
        vfs: &str,
    ) -> Result<Self> {
        let path = path.as_ref().to_owned();
        let vfs = vfs.to_owned();
        start(move || rusqlite::Connection::open_with_flags_and_vfs(path, flags, &vfs))
            .await
            .map_err(ConnectionError::Rusqlite)
    }

    /// Open a new connection to an in-memory SQLite database.
    ///
    /// [Database Connection](http://www.sqlite.org/c3ref/open.html) for a
    /// description of valid flag combinations.
    ///
    /// # Failure
    ///
    /// Will return `Err` if the underlying SQLite open call fails.
    pub async fn open_in_memory_with_flags(flags: OpenFlags) -> Result<Self> {
        start(move || rusqlite::Connection::open_in_memory_with_flags(flags))
            .await
            .map_err(ConnectionError::Rusqlite)
    }

    /// Open a new connection to an in-memory SQLite database using the
    /// specific flags and vfs name.
    ///
    /// [Database Connection](http://www.sqlite.org/c3ref/open.html) for a
    /// description of valid flag combinations.
    ///
    /// # Failure
    ///
    /// Will return `Err` if `vfs` cannot be converted to a C-compatible
    /// string or if the underlying SQLite open call fails.
    pub async fn open_in_memory_with_flags_and_vfs(flags: OpenFlags, vfs: &str) -> Result<Self> {
        let vfs = vfs.to_owned();
        start(move || rusqlite::Connection::open_in_memory_with_flags_and_vfs(flags, &vfs))
            .await
            .map_err(ConnectionError::Rusqlite)
    }

    /// Call a function in background thread and get the result
    /// asynchronously.
    ///
    /// # Failure
    ///
    /// Will return `Err` if the database connection has been closed.
    pub async fn call<F, R>(&self, function: F) -> Result<R>
    where
        F: FnOnce(&mut rusqlite::Connection) -> Result<R> + 'static + Send,
        R: Send + 'static,
    {
        let (sender, receiver) = oneshot::channel::<Result<R>>();

        self.sender
            .send(Message::Execute(Box::new(move |conn| {
                let value = function(conn);
                let _ = sender.send(value);
            })))
            .map_err(|_| ConnectionError::ConnectionClosed)?;

        receiver.await.map_err(|_| ConnectionError::ConnectionClosed)?
    }

    /// Call a function in background thread and get the result
    /// asynchronously.
    ///
    /// This method can cause a `panic` if the underlying database connection is closed.
    /// it is a more user-friendly alternative to the [`Connection::call`] method.
    /// It should be safe if the connection is never explicitly closed (using the [`Connection::close`] call).
    ///
    /// Calling this on a closed connection will cause a `panic`.
    pub async fn call_unwrap<F, R>(&self, function: F) -> R
    where
        F: FnOnce(&mut rusqlite::Connection) -> R + Send + 'static,
        R: Send + 'static,
    {
        let (sender, receiver) = oneshot::channel::<R>();

        self.sender
            .send(Message::Execute(Box::new(move |conn| {
                let value = function(conn);
                let _ = sender.send(value);
            })))
            .expect("database connection should be open");

        receiver.await.expect(BUG_TEXT)
    }

    /// Close the database connection.
    ///
    /// This is functionally equivalent to the `Drop` implementation for
    /// `Connection`. It consumes the `Connection`, but on error returns it
    /// to the caller for retry purposes.
    ///
    /// If successful, any following `close` operations performed
    /// on `Connection` copies will succeed immediately.
    ///
    /// On the other hand, any calls to [`Connection::call`] will return a [`Error::ConnectionClosed`],
    /// and any calls to [`Connection::call_unwrap`] will cause a `panic`.
    ///
    /// # Failure
    ///
    /// Will return `Err` if the underlying SQLite close call fails.
    pub async fn close(self) -> Result<()> {
        let (sender, receiver) = oneshot::channel::<std::result::Result<(), rusqlite::Error>>();

        if let Err(crossbeam::channel::SendError(_)) = self.sender.send(Message::Close(sender)) {
            // If the channel is closed on the other side, it means the connection closed successfully
            // This is a safeguard against calling close on a `Copy` of the connection
            return Ok(());
        }

        let result = receiver.await;

        if result.is_err() {
            // If we get a RecvError at this point, it also means the channel closed in the meantime
            // we can assume the connection is closed
            return Ok(());
        }

        result.unwrap().map_err(|e| ConnectionError::Close((self, e)))
    }
}

impl Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Connection").finish()
    }
}

impl From<rusqlite::Connection> for Connection {
    fn from(conn: rusqlite::Connection) -> Self {
        let (sender, receiver) = crossbeam::channel::unbounded::<Message>();
        thread::spawn(move || event_loop(conn, receiver));

        Self { sender }
    }
}

async fn start<F>(open: F) -> rusqlite::Result<Connection>
where
    F: FnOnce() -> rusqlite::Result<rusqlite::Connection> + Send + 'static,
{
    let (sender, receiver) = crossbeam::channel::unbounded::<Message>();
    let (result_sender, result_receiver) = oneshot::channel();

    thread::spawn(move || {
        let conn = match open() {
            Ok(c) => c,
            Err(e) => {
                let _ = result_sender.send(Err(e));
                return;
            }
        };

        if let Err(_e) = result_sender.send(Ok(())) {
            return;
        }

        event_loop(conn, receiver);
    });

    result_receiver
        .await
        .expect(BUG_TEXT)
        .map(|_| Connection { sender })
}

fn event_loop(mut conn: rusqlite::Connection, receiver: Receiver<Message>) {
    while let Ok(message) = receiver.recv() {
        match message {
            Message::Execute(f) => f(&mut conn),
            Message::Close(s) => {
                let result = conn.close();

                match result {
                    Ok(v) => {
                        s.send(Ok(v)).expect(BUG_TEXT);
                        break;
                    }
                    Err((c, e)) => {
                        conn = c;
                        s.send(Err(e)).expect(BUG_TEXT);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;
    use super::Result;

    use crate::*;

    #[tokio::test]
    async fn open_in_memory_test() -> Result<()> {
        let conn = Connection::open_in_memory().await;
        assert!(conn.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn call_success_test() -> Result<()> {
        let conn = Connection::open_in_memory().await?;

        let result = conn
            .call(|conn| {
                conn.execute(
                "CREATE TABLE person(id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL);",
                [],
            )
            .map_err(|e| e.into())
            })
            .await;

        assert_eq!(0, result.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn call_unwrap_success_test() -> Result<()> {
        let conn = Connection::open_in_memory().await?;

        let result = conn
            .call_unwrap(|conn| {
                conn.execute(
                "CREATE TABLE person(id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL);",
                [],
            )
            .unwrap()
            })
            .await;

        assert_eq!(0, result);

        Ok(())
    }

    #[tokio::test]
    async fn call_failure_test() -> Result<()> {
        let conn = Connection::open_in_memory().await?;

        let result = conn
            .call(|conn| conn.execute("Invalid sql", []).map_err(|e| e.into()))
            .await;

        assert!(match result.unwrap_err() {
            ConnectionError::Rusqlite(e) => {
                e == rusqlite::Error::SqlInputError {
                    error: rusqlite::ffi::Error {
                        code: rusqlite::ErrorCode::Unknown,
                        extended_code: 1,
                    },
                    msg: "near \"Invalid\": syntax error".to_string(),
                    sql: "Invalid sql".to_string(),
                    offset: 0,
                }
            }
            _ => false,
        });

        Ok(())
    }

    #[tokio::test]
    async fn close_success_test() -> Result<()> {
        let conn = Connection::open_in_memory().await?;

        assert!(conn.close().await.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn double_close_test() -> Result<()> {
        let conn = Connection::open_in_memory().await?;

        let conn2 = conn.clone();

        assert!(conn.close().await.is_ok());
        assert!(conn2.close().await.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn close_call_test() -> Result<()> {
        let conn = Connection::open_in_memory().await?;

        let conn2 = conn.clone();

        assert!(conn.close().await.is_ok());

        let result = conn2
            .call(|conn| conn.execute("SELECT 1;", []).map_err(|e| e.into()))
            .await;

        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::ConnectionClosed
        ));

        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn close_call_unwrap_test() {
        let conn = Connection::open_in_memory().await.unwrap();

        let conn2 = conn.clone();

        assert!(conn.close().await.is_ok());

        conn2
            .call_unwrap(|conn| conn.execute("SELECT 1;", []))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn close_failure_test() -> Result<()> {
        let conn = Connection::open_in_memory().await?;

        conn.call(|conn| {
            conn.execute(
                "CREATE TABLE person(id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL);",
                [],
            )
            .map_err(|e| e.into())
        })
        .await?;

        conn.call(|conn| {
            // Leak a prepared statement to make the database uncloseable
            // See https://www.sqlite.org/c3ref/close.html for details regarding this behaviour
            let stmt = Box::new(conn.prepare("INSERT INTO person VALUES (1, ?1);").unwrap());
            Box::leak(stmt);
            Ok(())
        })
        .await?;

        assert!(match conn.close().await.unwrap_err() {
            ConnectionError::Close((_, e)) => {
                e == rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error {
                        code: rusqlite::ErrorCode::DatabaseBusy,
                        extended_code: 5,
                    },
                    Some(
                        "unable to close due to unfinalized statements or unfinished backups"
                            .to_string(),
                    ),
                )
            }
            _ => false,
        });

        Ok(())
    }

    #[tokio::test]
    async fn debug_format_test() -> Result<()> {
        let conn = Connection::open_in_memory().await?;

        assert_eq!("Connection".to_string(), format!("{conn:?}"));

        Ok(())
    }

    #[tokio::test]
    async fn test_error_display() -> Result<()> {
        let conn = Connection::open_in_memory().await?;

        let error = ConnectionError::Close((conn, rusqlite::Error::InvalidQuery));
        assert_eq!(
            "Close((Connection, \"Query is not read-only\"))",
            format!("{error}")
        );

        let error = ConnectionError::ConnectionClosed;
        assert_eq!("ConnectionClosed", format!("{error}"));

        let error = ConnectionError::Rusqlite(rusqlite::Error::InvalidQuery);
        assert_eq!("Rusqlite(\"Query is not read-only\")", format!("{error}"));

        Ok(())
    }

    #[tokio::test]
    async fn test_error_source() -> Result<()> {
        let conn = Connection::open_in_memory().await?;

        let error = ConnectionError::Close((conn, rusqlite::Error::InvalidQuery));
        assert_eq!(
            std::error::Error::source(&error)
                .and_then(|e| e.downcast_ref::<rusqlite::Error>())
                .unwrap(),
            &rusqlite::Error::InvalidQuery,
        );

        let error = ConnectionError::ConnectionClosed;
        assert_eq!(
            std::error::Error::source(&error).and_then(|e| e.downcast_ref::<rusqlite::Error>()),
            None,
        );

        let error = ConnectionError::Rusqlite(rusqlite::Error::InvalidQuery);
        assert_eq!(
            std::error::Error::source(&error)
                .and_then(|e| e.downcast_ref::<rusqlite::Error>())
                .unwrap(),
            &rusqlite::Error::InvalidQuery,
        );

        Ok(())
    }

    fn failable_func(_: &rusqlite::Connection) -> std::result::Result<(), MyError> {
        Err(MyError::MySpecificError)
    }

    #[tokio::test]
    async fn test_ergonomic_errors() -> Result<()> {
        let conn = Connection::open_in_memory().await?;

        let res = conn
            .call(|conn| failable_func(conn).map_err(|e| ConnectionError::Other(Box::new(e))))
            .await
            .unwrap_err();

        let err = std::error::Error::source(&res)
            .and_then(|e| e.downcast_ref::<MyError>())
            .unwrap();

        assert!(matches!(err, MyError::MySpecificError));

        Ok(())
    }

    // The rest is boilerplate, not really that important

    #[derive(Debug)]
    enum MyError {
        MySpecificError,
    }

    impl Display for MyError {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Ok(())
        }
    }

    impl std::error::Error for MyError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            None
        }
    }
}
