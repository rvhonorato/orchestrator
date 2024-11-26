use crate::utils;
use axum::http::StatusCode;
use axum::{body::Bytes, BoxError};
use futures::Stream;
use futures::TryStreamExt;
use std::io;
use std::path::PathBuf;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
pub const UPLOADS_DIRECTORY: &str = "uploads";

// // to prevent directory traversal attacks we ensure the path consists of exactly one normal
// // component
// pub fn path_is_valid(path: &str) -> bool {
//     let path = std::path::Path::new(path);
//     let mut components = path.components().peekable();
//
//     if let Some(first) = components.peek() {
//         if !matches!(first, std::path::Component::Normal(_)) {
//             return false;
//         }
//     }
//
//     components.count() == 1
// }

pub async fn stream_to_file<S, E>(path: &PathBuf, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    // if !utils::path_is_valid(path) {
    //     return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    // }

    async {
        // Convert the stream into an AsyncRead.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. File implements AsyncWrite.
        let mut file = BufWriter::new(File::create(path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}
