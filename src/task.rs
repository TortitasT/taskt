use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    pub text: String,
    pub completed: bool,
}
impl Task {
    pub fn new(text: String) -> Self {
        Self {
            text,
            completed: false,
        }
    }
}
