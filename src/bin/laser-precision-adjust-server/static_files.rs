use std::{collections::HashMap, io::Cursor};

use axum::{
    body::StreamBody,
    extract::Path,
    http::{header, StatusCode},
    response::IntoResponse,
};

use maplit::hashmap;

use mime_guess::mime;
use regex::Regex;
use tokio_util::io::ReaderStream;
use typescript_converter_macro::include_ts_relative;

trait IntoBody<T> {
    fn into_body(self) -> StreamBody<ReaderStream<Cursor<T>>>;
}

impl IntoBody<&'static str> for &'static str {
    fn into_body(self) -> StreamBody<ReaderStream<Cursor<&'static str>>> {
        let stream = Cursor::new(self);
        let stream = ReaderStream::new(stream);
        StreamBody::new(stream)
    }
}

impl IntoBody<&'static [u8]> for &'static [u8] {
    fn into_body(self) -> StreamBody<ReaderStream<Cursor<&'static [u8]>>> {
        let stream = Cursor::new(self);
        let stream = ReaderStream::new(stream);
        StreamBody::new(stream)
    }
}

lazy_static::lazy_static! {
    static ref JS_REMOVE_EXT: Regex = Regex::new(r"\..+").unwrap();

    // js with map
    static ref JS_DATA: HashMap<&'static str, (&'static str, &'static str)> = hashmap! {
        "common" => include_ts_relative!("wwwroot/ts/common.ts"),
    };

    // css
    static ref CSS_DATA: HashMap<&'static str, &'static str> = hashmap! {
        "site.css" => include_str!("wwwroot/css/site.css"),
    };

    // images
    static ref IMAGE_DATA: HashMap<&'static str, (&'static [u8], &'static str)> = hashmap! {
        "favicon.ico" => (include_bytes!("wwwroot/images/favicon.ico").as_ref(), mime::IMAGE.as_ref()),
    };
}

#[iftree::include_file_tree(
    "
paths = '**'
base_folder = 'src/bin/laser-precision-adjust-server/wwwroot/lib/'
"
)]
pub struct LibraryAsset {
    contents_bytes: &'static [u8],
    relative_path: &'static str,
}

/// Handle static files: js, css, images, etc.
pub(crate) async fn handle_static(Path((path, file)): Path<(String, String)>) -> impl IntoResponse {
    let not_found = StatusCode::NOT_FOUND.into_response();

    match path.as_str() {
        "js" => {
            let name = JS_REMOVE_EXT.replace_all(&file, "");
            match JS_DATA.get(name.as_ref()) {
                Some((js, map)) => {
                    if file.ends_with(".map") {
                        let headers = [(header::CONTENT_TYPE, mime::TEXT_PLAIN_UTF_8.as_ref())];
                        (headers, map.into_body()).into_response()
                    } else {
                        let headers = [(header::CONTENT_TYPE, mime::APPLICATION_JAVASCRIPT_UTF_8.as_ref())];
                        (headers, js.into_body()).into_response()
                    }
                }
                None => not_found,
            }
        }
        "css" => CSS_DATA.get(file.as_str()).map_or(not_found, |css| {
            let headers = [(header::CONTENT_TYPE, mime::TEXT_CSS_UTF_8.as_ref())];
            (headers,  css.into_body()).into_response()
        }),
        "images" => IMAGE_DATA.get(file.as_str()).map_or(not_found, |image| {
            let headers = [(header::CONTENT_TYPE, image.1)];

            (headers, image.0.into_body()).into_response()
        }),
        _ => not_found,
    }
}

/// Handle library files: js, css, images, etc.
pub(crate) async fn handle_lib(Path(path): Path<String>) -> impl IntoResponse {
    ASSETS
        .iter()
        .find(|asset| asset.relative_path == path.as_str())
        .map_or(StatusCode::NOT_FOUND.into_response(), |asset| {
            let mime_type = mime_guess::from_path(asset.relative_path).first_or_octet_stream();
            let headers = [(header::CONTENT_TYPE, mime_type.as_ref())];

            (headers, asset.contents_bytes.into_body()).into_response()
        })
}
    