use crate::models::status_dto::Status;
use crate::services::client::ClientError;
use std::fs;
use std::path::PathBuf;
use utoipa::ToSchema;

#[derive(serde::Serialize, Debug, ToSchema)]
pub struct Payload {
    pub id: u32,
    input: Vec<u8>,
    filename: String,
    ouput: Option<Vec<u8>>,
    pub status: Status,
    #[schema(value_type = String)]
    pub loc: PathBuf,
}

impl Payload {
    pub fn new() -> Payload {
        Payload {
            id: 0,
            input: vec![],
            filename: String::new(),
            ouput: None,
            status: Status::Unknown,
            loc: PathBuf::new(),
        }
    }

    pub fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    pub fn set_filename(&mut self, filename: String) {
        self.filename = filename;
    }

    pub fn set_input(&mut self, input: Vec<u8>) {
        self.input = input;
    }

    // pub fn set_output(&mut self, output: Vec<u8>) {
    //     self.ouput = Some(output);
    // }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn set_loc(&mut self, loc: PathBuf) {
        self.loc = loc;
    }

    pub fn prepare(&mut self, data_path: &str) -> Result<(), std::io::Error> {
        self.loc = std::path::Path::new(&data_path).join(self.id.to_string());

        // Create directory dor this payload
        fs::create_dir_all(&self.loc)?;

        // Dump data to this directory
        fs::write(self.loc.join(&self.filename), &self.input)?;

        Ok(())
    }

    pub fn execute(&mut self) -> Result<(), ClientError> {
        // todo!();

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_set_filename() {
        let mut p = Payload::new();
        assert_eq!(p.filename, "");
        p.set_filename("test.txt".to_string());
        assert_eq!(p.filename, "test.txt");
    }

    #[tokio::test]
    async fn test_set_input() {
        let mut p = Payload::new();
        assert_eq!(p.input.len(), 0);
        let data = b"Hello, world!".to_vec();
        p.set_input(data.clone());
        assert_eq!(p.input, data);
    }

    #[tokio::test]
    async fn test_prepare() {
        let mut p = Payload::new();
        p.id = 1;
        p.set_filename("test.txt".to_string());
        p.set_input(b"Test data".to_vec());

        let temp_dir = tempfile::tempdir().unwrap();
        let data_path = temp_dir.path().to_str().unwrap();

        let result = p.prepare(data_path);
        assert!(result.is_ok());

        let expected_path = temp_dir.path().join("1").join("test.txt");
        assert!(expected_path.exists());

        let content = fs::read_to_string(expected_path).unwrap();
        assert_eq!(content, "Test data");
    }
}
