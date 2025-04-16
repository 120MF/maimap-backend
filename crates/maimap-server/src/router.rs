use crate::handler::arcade::{get_arcade_by_id_handler, search_arcades_handler};
use salvo::Router;

pub fn router() -> Router {
    Router::with_path("arcades")
        .get(search_arcades_handler)
        .push(
            Router::with_path("{arcade_id}")
                .get(get_arcade_by_id_handler)
                .push(
                    Router::with_path("comments").get(crate::handler::arcade::get_comments_handler),
                )
                .push(Router::with_path("tags").get(crate::handler::arcade::get_tags_handler)),
        )
}
