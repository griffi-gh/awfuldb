use tokio::io::{AsyncRead, AsyncWrite};

use crate::shape::DatabaseShape;

pub trait AsyncRw: AsyncRead + AsyncWrite {}
impl<T: AsyncRead + AsyncWrite> AsyncRw for T {}

struct DatabaseData<T: AsyncRw> {
  file: T,
}

pub struct Database<T: AsyncRw> {
  data: DatabaseData<T>,
  shape: DatabaseShape,
}
