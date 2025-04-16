mod get_by_id;
mod get_comments;
mod get_tags;
mod search;

pub use get_by_id::get_arcade_by_id_handler;
pub use get_comments::get_comments_handler;
pub use get_tags::get_tags_handler;
pub use search::search_arcades_handler;
