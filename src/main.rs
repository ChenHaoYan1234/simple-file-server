use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use actix_files::NamedFile;
use actix_web::{
    App, Error, HttpServer, get,
    http::header::{ContentDisposition, DispositionType},
    middleware,
};
use log::error;

static FILE: OnceLock<PathBuf> = OnceLock::new();

#[get("/")]
async fn index() -> Result<NamedFile, Error> {
    let file = NamedFile::open_async(FILE.get().unwrap()).await?;

    Ok(file
        .use_etag(true)
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Inline,
            parameters: vec![],
        }))
}

#[actix_web::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let args = std::env::args().collect::<Vec<String>>();

    // Check if the user provided the required arguments
    if args.len() < 4 {
        error!("Usage: {} <port> <path> <workers>", args[0]);
        std::process::exit(1);
    }

    let port = args[1].parse::<u16>().unwrap_or_else(|_| {
        error!("Invalid port number");
        std::process::exit(1);
    });
    let path = Path::new(&args[2]);
    let workers = args[3].parse::<usize>().unwrap_or_else(|_| {
        error!("Invalid number of workers");
        std::process::exit(1);
    });

    if !path.exists() || path.is_dir() {
        error!("Path does not exist or is a directory");
        std::process::exit(1);
    }

    FILE.get_or_init(|| path.to_path_buf());

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(index)
    })
    .workers(workers)
    .bind(("127.0.0.1", port))
    .unwrap_or_else(|_| {
        error!("Failed to bind to port");
        std::process::exit(1);
    })
    .run()
    .await
    .unwrap_or_else(|_| {
        error!("Failed to run server");
        std::process::exit(1);
    });
}
