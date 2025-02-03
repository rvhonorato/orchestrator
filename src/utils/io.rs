use axum::http::StatusCode;
use axum::{body::Bytes, BoxError};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use futures::Stream;
use futures::TryStreamExt;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;

pub async fn stream_to_file<S, E>(filename: PathBuf, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    async {
        // Convert the stream into an AsyncRead.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. File implements AsyncWrite.
        let mut file = BufWriter::new(File::create(filename).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

pub fn stream_file_to_base64(path: &str) -> io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut buffer = [0; 3072]; // read in chunks of 3072
    let mut base64 = String::new();
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        base64.push_str(&STANDARD.encode(&buffer[..bytes_read]));
    }
    Ok(base64)
}

pub fn base64_to_file(base64_content: &str, output_path: PathBuf) -> io::Result<()> {
    let decoded_bytes = STANDARD
        .decode(base64_content)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid base64"))?;

    std::fs::create_dir_all(output_path.parent().unwrap())?;
    std::fs::write(&output_path, &decoded_bytes)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_stream_to_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let stream = tokio_stream::iter(vec![Ok::<bytes::Bytes, BoxError>(Bytes::from_static(
            b"hello",
        ))]);

        let result = stream_to_file(file_path.clone(), stream).await;
        assert!(result.is_ok());

        let content = tokio::fs::read_to_string(file_path).await.unwrap();
        assert_eq!(content, "hello")
    }

    #[test]
    fn test_stream_file_to_base64() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let _ = temp_file.write_all(b"hello");
        let temp_path = temp_file.path().to_str().unwrap();
        let result = stream_file_to_base64(temp_path).unwrap();
        assert_eq!(result, "aGVsbG8=")
    }

    #[test]
    fn test_base64_to_file_success() {
        // Create a temporary directory for the test
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let base64_content = "SGVsbG8gV29ybGQh"; // "Hello World!" in base64

        let result = base64_to_file(base64_content, file_path.clone());

        assert!(result.is_ok());

        let file_content = fs::read_to_string(&file_path).expect("Failed to read file");
        assert_eq!(file_content, "Hello World!");
    }

    #[test]
    fn test_base64_to_file_invalid_base64() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let base64_content = "InvalidBase64!!";

        let result = base64_to_file(base64_content, file_path);

        assert!(result.is_err());

        if let Err(e) = result {
            assert_eq!(e.kind(), io::ErrorKind::InvalidData);
        }
    }

    #[test]
    fn test_base64_to_file_directory_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let base64_content = "SGVsbG8gV29ybGQh"; // "Hello World!" in base64

        let result = base64_to_file(base64_content, file_path.clone());

        assert!(result.is_ok());

        assert!(file_path.parent().unwrap().exists());

        let file_content = fs::read_to_string(&file_path).expect("Failed to read file");
        assert_eq!(file_content, "Hello World!");
    }
}
