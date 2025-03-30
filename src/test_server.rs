use axum::{Router, extract::Path};
use http::StatusCode;
use tokio::net::TcpListener;

pub const PATHS: phf::Map<&str, TestPath> = phf::phf_map! {
    "ok" => TestPath {
        code: StatusCode::OK,
        length: 10,
    },
    "201" => TestPath {
        code: StatusCode::CREATED,
        length: 11,
    },
};

pub struct TestPath {
    pub code: StatusCode,
    pub length: usize,
}

pub fn launch() -> u16 {
    let listener = std::net::TcpListener::bind("[::1]:0").unwrap();
    listener.set_nonblocking(true).unwrap();

    let port = listener.local_addr().unwrap().port();
    let app = make_router();

    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime")
            .block_on(async {
                let listener = TcpListener::from_std(listener).unwrap();

                axum::serve(listener, app).await.unwrap();
            })
    });

    port
}

fn make_router() -> Router {
    use axum::routing::get;

    Router::new()
        .route("/", get(|| async { "OK" }))
        .route("/{*path}", get(get_test_path))
}

async fn get_test_path(Path(path): Path<String>) -> (StatusCode, String) {
    dbg!(&path);
    let Some(params) = PATHS.get(&path) else {
        return (StatusCode::NOT_FOUND, "Not found".into());
    };
    (params.code, "A".repeat(params.length))
}

mod test {
    use super::*;
    use curl::easy::Easy;

    #[test]
    fn server_startup() {
        let port = launch();

        let mut easy = Easy::new();
        easy.url(&format!("http://localhost:{port}")).unwrap();
        easy.perform().unwrap();
        assert_eq!(easy.response_code().unwrap(), 200);
        assert_eq!(easy.content_length_download().unwrap(), 2.0);

        easy.url(&format!("http://localhost:{port}/ok")).unwrap();
        easy.perform().unwrap();
        assert_eq!(easy.response_code().unwrap(), 200);
        assert_eq!(easy.content_length_download().unwrap(), 10.0);

        easy.url(&format!("http://localhost:{port}/notfound"))
            .unwrap();
        easy.perform().unwrap();
        assert_eq!(easy.response_code().unwrap(), 404);
        assert_eq!(easy.content_length_download().unwrap(), 9.0);

        easy.url(&format!("http://localhost:{port}/201")).unwrap();
        easy.perform().unwrap();
        assert_eq!(easy.response_code().unwrap(), 201);
        assert_eq!(easy.content_length_download().unwrap(), 11.0);
    }
}
