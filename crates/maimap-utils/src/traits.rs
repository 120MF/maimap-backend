use serde_json::Value;

pub trait ToResponse {
    fn to_response(&self) -> Value;
}

impl<T: ToResponse> ToResponse for Vec<T> {
    fn to_response(&self) -> Value {
        Value::Array(self.iter().map(|item| item.to_response()).collect())
    }
}
