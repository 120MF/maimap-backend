use crate::errors::AppError;
use crate::res::ApiResponse;
use anyhow::Error;
use salvo::prelude::*;
pub fn handle_error(res: &mut Response, err: Error) {
    if let Some(app_err) = err.downcast_ref::<AppError>() {
        match app_err {
            AppError::Validation(_) => res.status_code(StatusCode::BAD_REQUEST),
            _ => res.status_code(StatusCode::INTERNAL_SERVER_ERROR),
        };
    } else {
        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    }

    res.render(Json(ApiResponse::<()>::error(err.to_string())));
}
