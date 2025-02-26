#![feature(
    decl_macro,
    iter_array_chunks,
    str_as_str
)]


pub mod packet;

pub use diff_match_patch_rs as dmp;
pub use uuid::Uuid;
