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

// fn to_base64(inp: &str) -> String {
//     STANDARD.encode(inp)
// }
//
// fn from_base64(inp: String) -> String {
//     match STANDARD.decode(inp) {
//         Ok(v) => String::from_utf8(v).expect("Could not decode"),
//         Err(_) => "".to_string(),
//     }
// }
//
// pub fn is_zip(path: &str) -> bool {
//     if let Ok(mut file) = std::fs::File::open(path) {
//         let mut buffer = [0u8; 4];
//         if file.read_exact(&mut buffer).is_ok() {
//             return &buffer == b"PK\x03\x04";
//         }
//     }
//     false
// }

#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_stream_file_to_base64() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let _ = temp_file.write_all(b"hello");
        let temp_path = temp_file.path().to_str().unwrap();
        let result = stream_file_to_base64(temp_path).unwrap();
        assert_eq!(result, "aGVsbG8=")
    }

    // #[test]
    // fn test_to_base64() {
    //     let result = to_base64("hello");
    //     assert_eq!(result, "aGVsbG8=")
    // }

    // #[test]
    // fn test_from_base64() {
    //     let result = from_base64("aGVsbG8=".to_string());
    //     assert_eq!(result, "hello")
    // }

    // #[test]
    // fn test_is_zip_false() {
    //     let mut temp_file = NamedTempFile::new().unwrap();
    //     let _ = temp_file.write_all(b"hello");
    //     let temp_path = temp_file.path().to_str().unwrap();
    //     let result = is_zip(temp_path);
    //     assert!(!result)
    // }

    // #[test]
    // fn test_is_zip_true() {
    //     let result = is_zip("tests/file.zip");
    //     assert!(result)
    // }
}
