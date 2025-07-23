use crate::models::status_dto::Status;
use crate::utils::io::stream_to_file;
use axum::http::StatusCode;
use axum::{body::Bytes, BoxError};
use futures::Stream;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(serde::Serialize, Debug, ToSchema)]
pub struct Job {
    pub id: i32,
    pub user_id: i32,
    pub service: String,
    pub status: Status,
    #[schema(value_type = String)]
    pub loc: PathBuf,
    pub dest_id: String,
}

impl Job {
    pub fn new(data_path: &str) -> Job {
        let loc = std::path::Path::new(&data_path).join(Uuid::new_v4().to_string());
        Job {
            id: 0,
            user_id: 0,
            service: String::new(),
            status: Status::Unknown,
            loc,
            dest_id: String::new(),
        }
    }

    pub async fn save_to_disk<S, E>(
        &mut self,
        stream: S,
        filename: &String,
    ) -> Result<(), (StatusCode, String)>
    where
        S: Stream<Item = Result<Bytes, E>>,
        E: Into<BoxError>,
    {
        match fs::create_dir(&self.loc) {
            Ok(_) => (),
            Err(e) => println!("could not create directory {}", e),
        }
        let full_path = std::path::Path::join(&self.loc, filename);
        stream_to_file(full_path, stream).await?;
        Ok(())
    }

    pub fn download(self) -> Vec<u8> {
        let mut file = fs::File::open(self.loc.join("output.zip")).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        buffer
    }

    pub fn remove_from_disk(&self) -> Result<(), std::io::Error> {
        fs::remove_dir_all(&self.loc)
    }

    pub fn set_service(&mut self, service: String) {
        self.service = service
    }

    pub fn set_user_id(&mut self, user_id: i32) {
        self.user_id = user_id;
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use bytes::Bytes;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_to_disk() {
        let tempdir = TempDir::new().unwrap();
        let mut job = Job::new(tempdir.path().to_str().unwrap());

        let stream = tokio_stream::iter(vec![Ok::<bytes::Bytes, BoxError>(Bytes::from_static(
            b"hello",
        ))]);
        let filename = String::from("test.txt");

        let result = job.save_to_disk(stream, &filename).await;
        assert!(result.is_ok());

        let content = tokio::fs::read_to_string(job.loc.join(&filename))
            .await
            .unwrap();
        assert_eq!(content, "hello");
    }

    #[tokio::test]
    async fn test_download() {
        let tempdir = TempDir::new().unwrap();
        let job = Job::new(tempdir.path().to_str().unwrap());

        let _ = fs::create_dir_all(&job.loc);
        let test_data = b"test content".to_vec();
        fs::write(job.loc.join("output.zip"), &test_data).unwrap();

        let result = job.download();
        assert_eq!(result, test_data);
    }

    #[test]
    fn test_remove_from_disk() {
        let tempdir = TempDir::new().unwrap();
        let job = Job::new(tempdir.path().to_str().unwrap());

        // First verify the directory exists
        fs::create_dir_all(&job.loc).unwrap();
        assert!(Path::new(&job.loc).exists());

        // Remove the directory
        let _ = job.remove_from_disk();

        // Verify the directory no longer exists
        assert!(!Path::new(&job.loc).exists());
    }

    #[test]
    fn test_set_service() {
        let mut job = Job::new("");
        job.set_service("test".to_string());
        assert_eq!(job.service, "test".to_string())
    }

    #[test]
    fn test_set_user_id() {
        let mut job = Job::new("");
        job.set_user_id(99);
        assert_eq!(job.user_id, 99)
    }
}
