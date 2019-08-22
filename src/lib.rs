//! ODBC support for the `r2d2` connection pool.
extern crate r2d2;
extern crate odbc;

#[macro_use]
extern crate lazy_static;

use std::error::Error;
use std::fmt;
use odbc::*;

#[derive(Debug)]
pub struct ODBCConnectionManager {
    connection_string: String
}

pub struct ODBCConnection<'a>(Connection<'a>);

unsafe impl Send for ODBCConnection<'static> {}

impl <'a> ODBCConnection<'a> {
    pub fn raw(&self) -> &Connection<'a> {
        &self.0
    }
}

pub struct ODBCEnv(Environment<Version3>);

unsafe impl Sync for ODBCEnv {}

unsafe impl Send for ODBCEnv {}

#[derive(Debug)]
pub struct ODBCError(Box<Error>);

lazy_static! {
    static ref ENV: ODBCEnv = ODBCEnv(create_environment_v3().unwrap());
}

impl Error for ODBCError {
    fn description(&self) -> &str {
        "Error connecting DB"
    }
}

impl fmt::Display for ODBCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<DiagnosticRecord> for ODBCError {
    fn from(err: DiagnosticRecord) -> Self {
        ODBCError(Box::new(err))
    }
}

impl <E: 'static> From<std::sync::PoisonError<E>> for ODBCError {
    fn from(err: std::sync::PoisonError<E>) -> Self {
        ODBCError(Box::new(err))
    }
}

impl ODBCConnectionManager {
    /// Creates a new `ODBCConnectionManager`.
    pub fn new<S: Into<String>>(connection_string: S) -> ODBCConnectionManager
    {
        ODBCConnectionManager {
            connection_string: connection_string.into()
        }
    }
}

impl r2d2::ManageConnection for ODBCConnectionManager {
    type Connection = ODBCConnection<'static>;
    type Error = ODBCError;

    fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        let env = &ENV.0;
        Ok(ODBCConnection(env.connect_with_connection_string(&self.connection_string)?))
    }

    fn is_valid(&self, _conn: &mut Self::Connection) -> std::result::Result<(), Self::Error> {
        match Statement::with_parent(_conn.raw()).unwrap().exec_direct("select 1") {
            Ok(_x) => {
                Ok(())
            },
            Err(e) => {
                Err(Self::Error::from(e))
            }
        }
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        //TODO
        false
    }
}
