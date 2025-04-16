use crate::res::ApiResponse;
use maimap_utils::errors::{AppError, Error, Result};
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

pub fn paginate_results<T: Clone>(
    results: &[T],
    page_index: Option<u32>,
    page_size: Option<u32>,
) -> Result<Vec<T>> {
    if page_index.is_some() != page_size.is_some() {
        return Err(AppError::Validation(
            "分页需要同时提供page_index、page_size两个参数".to_string(),
        )
        .into());
    }

    if let (Some(page_index), Some(page_size)) = (page_index, page_size) {
        if page_index < 1 || page_size < 1 {
            return Err(AppError::Validation("页码和每页大小必须大于0".to_string()).into());
        }
        let start = ((page_index - 1) * page_size) as usize;
        let end = std::cmp::min(start + page_size as usize, results.len());

        if start < results.len() {
            Ok(results[start..end].to_vec())
        } else {
            Ok(Vec::new())
        }
    } else {
        Ok(results.to_vec())
    }
}
