use crate::models::status_dto::Status;
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

    pub fn set_filename(&mut self, filename: String) {
        self.filename = filename;
    }

    pub fn set_input(&mut self, input: Vec<u8>) {
        self.input = input;
    }

    // pub fn set_output(&mut self, output: Vec<u8>) {
    //     self.ouput = Some(output);
    // }

    // pub fn set_status(&mut self, status: Status) {
    //     self.status = status;
    // }

    pub fn prepare(&mut self, data_path: &str) -> Result<(), std::io::Error> {
        self.loc = std::path::Path::new(&data_path).join(self.id.to_string());

        // Create directory dor this payload
        fs::create_dir_all(&self.loc)?;

        // Dump data to this directory
        fs::write(self.loc.join(&self.filename), &self.input)?;

        Ok(())
    }

    // pub fn execute(&mut self) {}
}
