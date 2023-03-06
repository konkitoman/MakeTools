use serde::Serialize;

#[derive(Serialize)]
pub struct Command {
    pub arguments: Vec<String>,
    pub directory: String,
    pub file: String,
    pub output: String,
}
