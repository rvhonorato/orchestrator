use axum::http::StatusCode;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

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
mod tests {}
