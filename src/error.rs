#[derive(Debug)]
pub enum Error {
    NotFound,
    BadRequest(String),
    UrldecodeError(serde_urlencoded::de::Error),
    SqliteError(rusqlite::Error),
    HyperHttpError(hyper::http::Error),
    HyperError(hyper::Error),
    UuidError(uuid::Error),
    TemplateError(tera::Error),
}

impl From<serde_urlencoded::de::Error> for Error {
    fn from(e: serde_urlencoded::de::Error) -> Error {
        Error::UrldecodeError(e)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Error {
        Error::SqliteError(e)
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Error {
        Error::HyperError(e)
    }
}

impl From<hyper::http::Error> for Error {
    fn from(e: hyper::http::Error) -> Error {
        Error::HyperHttpError(e)
    }
}

impl From<uuid::Error> for Error {
    fn from(e: uuid::Error) -> Error {
        Error::UuidError(e)
    }
}

impl From<tera::Error> for Error {
    fn from(e: tera::Error) -> Error {
        Error::TemplateError(e)
    }
}
