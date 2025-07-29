use axum::http::StatusCode;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

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

/// Sanitize filename to prevent path traversal attacks
pub fn sanitize_filename(filename: &str) -> String {
    std::path::Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string()
}

/// Save a multipart field to disk
pub async fn save_file(
    mut field: axum::extract::multipart::Field<'_>,
    path: &std::path::Path,
) -> Result<(), (StatusCode, String)> {
    let mut file = tokio::fs::File::create(path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("File creation failed: {e}"),
        )
    })?;

    let mut buffer = Vec::with_capacity(1024 * 1024); // 1MB buffer

    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Chunk read failed: {e}")))?
    {
        buffer.extend_from_slice(&chunk);

        // Write in chunks to balance memory and performance
        if buffer.len() >= 1024 * 1024 {
            file.write_all(&buffer).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Write failed: {e}"),
                )
            })?;
            buffer.clear();
        }
    }

    // Write remaining data
    if !buffer.is_empty() {
        file.write_all(&buffer).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Final write failed: {e}"),
            )
        })?;
    }

    file.flush().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Flush failed: {e}"),
        )
    })?;

    Ok(())
}

pub fn zip_directory(src_dir: &PathBuf, dst_file: &PathBuf) -> zip::result::ZipResult<()> {
    // Create the output file
    let file = File::create(dst_file)?;
    let mut zip = ZipWriter::new(file);

    // Set options for the zip file with explicit type annotation
    let options: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // Walk through the directory
    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    for entry in it.filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Ok(name) = path.strip_prefix(src_dir) {
            // Skip the root directory itself
            if name.as_os_str().is_empty() {
                continue;
            }

            // Convert path to string
            let name_str = name.to_str().ok_or_else(|| {
                zip::result::ZipError::Io(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid UTF-8 in file path",
                ))
            })?;

            if path.is_dir() {
                // Add directory entry
                zip.add_directory(name_str, options)?;
            } else {
                // Add file to the zip archive
                zip.start_file(name_str, options)?;
                let mut f = File::open(path)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            }
        } else {
            return Err(zip::result::ZipError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Path prefix mismatch",
            )));
        }
    }

    zip.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

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
