use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            count: None,
        }
    }

    pub fn with_count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }
    pub fn error(message: impl ToString) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
            count: None,
        }
    }
}
