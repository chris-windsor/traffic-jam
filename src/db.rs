use crate::create_pool;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use lazy_static::lazy_static;

type PgPool = Pool<ConnectionManager<PgConnection>>;

lazy_static! {
    pub static ref POOL: PgPool = create_pool();
}
