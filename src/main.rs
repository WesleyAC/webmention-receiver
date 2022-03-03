use chrono::TimeZone;
use error::Error;
use hyper::header::CONTENT_TYPE;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use rusqlite::params;
use serde::Deserialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use templates::templates;
use types::{Config, Webmention};
use url::Url;
use uuid::Uuid;

mod db;
mod error;
mod templates;
mod types;

async fn receive_webmention(domain: &str, request: Request<Body>) -> Result<Response<Body>, Error> {
    log::info!("Got webmention for {}", domain);

    if let Some(domains) = &CONFIG.allowed_domains {
        if !domains.contains(&domain.to_string()) {
            return Ok(Response::builder()
                .status(403)
                .body("The specified target domain is not allowed to use this server".into())?);
        }
    }

    if request.headers()[CONTENT_TYPE] != "application/x-www-form-urlencoded" {
        return Err(Error::BadRequest("Invalid Content-Type".into()));
    }

    let body = hyper::body::to_bytes(request.into_body()).await?;

    #[derive(Deserialize)]
    struct Request {
        source: Url,
        target: Url,
    }
    let data: Request = serde_urlencoded::from_bytes(&body)?;

    if !(data.source.scheme() == "https" || data.source.scheme() == "http")
        || !(data.target.scheme() == "https" || data.target.scheme() == "http")
    {
        return Err(Error::BadRequest("Invalid URL scheme".into()));
    }

    if data.source == data.target {
        return Err(Error::BadRequest(
            "source and target must be different".into(),
        ));
    }

    if data.target.host_str() != Some(domain) {
        return Err(Error::BadRequest("Invalid target domain".into()));
    }

    let now = chrono::Utc::now().timestamp_millis();
    let db = db::open()?;
    let id: Result<Uuid, rusqlite::Error> = db.query_row(
        "SELECT id FROM webmentions WHERE domain = ?1 AND source = ?2 AND target = ?3",
        params![domain, data.source, data.target],
        |row| Ok(row.get("id")?),
    );
    let id = match id {
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let id = Uuid::new_v4();
            log::info!("Inserting new webmention: {}", id);
            db.execute(
                r#"INSERT INTO webmentions
                    (id, domain, source, target, date_added, date_updated)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                params![id, domain, data.source, data.target, now, now],
            )?;
            id
        }
        Ok(id) => {
            log::info!("Updating existing webmention: {}", id);
            db.execute(
                r#"UPDATE webmentions
                    SET date_updated = ?1
                    WHERE domain = ?2 AND source = ?3 AND target = ?4
                "#,
                params![now, domain, data.source, data.target],
            )?;
            id
        }
        Err(error) => Err(error)?,
    };

    Ok(Response::builder()
        .status(201)
        .body(format!("{}/{}/mention/{}", CONFIG.external_url, domain, id).into())?)
}

fn show_webmention(domain: &str, mention: &str) -> Result<Response<Body>, Error> {
    let mention = Uuid::parse_str(mention)?;
    let db = db::open()?;
    let mention = db.query_row(
        "SELECT * FROM webmentions WHERE domain = ?1 AND id = ?2",
        params![domain, mention],
        |row| Ok(Webmention::try_from(row)?),
    )?;

    Ok(Response::new(format!("{:#?}", mention).into()))
}

fn show_domain(domain: &str, format: &str) -> Result<Response<Body>, Error> {
    let db = db::open()?;
    let mut stmt =
        db.prepare("SELECT * FROM webmentions WHERE domain = ?1 ORDER BY date_updated DESC")?;
    let mentions: Vec<Webmention> = stmt
        .query_map(params![domain], |row| Webmention::try_from(row))?
        .collect::<Result<Vec<Webmention>, rusqlite::Error>>()?;

    let mut context = tera::Context::new();
    let config: &Config = &CONFIG;
    context.insert("config", config);
    context.insert("domain", &domain);
    context.insert("mentions", &mentions);
    context.insert(
        "last_updated",
        &mentions
            .iter()
            .map(|x| &x.date_updated)
            .max()
            .unwrap_or(&types::Timestamp(chrono::Utc.timestamp_nanos(0))),
    );
    Ok(Response::builder()
        .status(200)
        .header(CONTENT_TYPE, format)
        .body(
            templates()
                .render(
                    match format {
                        "text/html" => "domain.html",
                        "application/xml" => "feed.xml",
                        _ => unreachable!(),
                    },
                    &context,
                )?
                .into(),
        )?)
}

fn robots_txt() -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(200)
        .body(include_str!("templates/robots.txt").into())?)
}

fn index() -> Result<Response<Body>, Error> {
    let mut context = tera::Context::new();
    let config: &Config = &CONFIG;
    context.insert("config", config);
    Ok(Response::builder()
        .status(200)
        .header(CONTENT_TYPE, "text/html")
        .body(templates().render("index.html", &context)?.into())?)
}

fn assets(request: Request<Body>) -> Result<Response<Body>, Error> {
    #[derive(rust_embed::RustEmbed)]
    #[folder = "src/assets/"]
    #[prefix = "/assets/"]
    struct Assets;
    let mime = mime_guess::from_path(request.uri().path()).first_or_octet_stream();
    let file = Assets::get(request.uri().path());
    match file {
        Some(file) => Ok(Response::builder()
            .header(CONTENT_TYPE, mime.as_ref())
            .body(Body::from(file.data))?),
        None => Err(Error::NotFound),
    }
}

async fn request_handler(request: Request<Body>) -> Result<Response<Body>, Error> {
    log::debug!("{:?} {:?}", request.method(), request.uri());
    let uri = request.uri().clone();
    let uri: Vec<&str> = uri.path().trim_matches('/').split("/").collect();
    match (request.method(), &uri[..]) {
        (&Method::GET, &[""]) => index(),
        (&Method::GET, &["robots.txt"]) => robots_txt(),
        (&Method::GET, &[domain]) => show_domain(domain, "text/html"),
        (&Method::GET, &[domain, "feed.xml"]) => show_domain(domain, "application/xml"),
        (&Method::GET, &[domain, "mention", mention]) => show_webmention(domain, mention),
        (&Method::POST, &[domain, "receiver"]) => receive_webmention(domain, request).await,
        (&Method::GET, &["assets", ..]) => assets(request),
        _ => Ok(make_404()),
    }
}

fn make_404() -> Response<Body> {
    Response::builder()
        .status(404)
        .body(format!("404 not found").into())
        .unwrap()
}

async fn handle_errors(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    match request_handler(request).await {
        Ok(response) => Ok(response),
        Err(Error::NotFound) => Ok(make_404()),
        Err(err) => {
            log::error!("Error: {:?}", err);
            let status = match err {
                Error::BadRequest(_) => 400,
                Error::UrldecodeError(_) => 400,
                Error::UuidError(_) => 404,
                _ => 500,
            };
            Ok(Response::builder()
                .status(status)
                .body(format!("{:#?}", err).into())
                .unwrap())
        }
    }
}

lazy_static::lazy_static! {
    static ref CONFIG: Config = toml::from_str(&std::fs::read_to_string("config.toml").unwrap()).unwrap();
}

#[tokio::main]
async fn main() {
    env_logger::init();

    db::run_migrations(&db::open().unwrap()).unwrap();

    let addr = CONFIG
        .bind
        .unwrap_or(SocketAddr::from(([127, 0, 0, 1], 28081)));

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_errors)) });

    let server = Server::bind(&addr).serve(make_svc);

    server.await.unwrap();
}
