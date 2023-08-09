use std::{ops::Range, mem::size_of};
use anyhow::Result;
use rkyv::{Archive, Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncSeekExt};
use crate::shape::{DatabaseShape, ArchivedDatabaseShape};

pub trait RwData: AsyncReadExt + AsyncWriteExt + AsyncSeekExt + Unpin {}
impl<T: AsyncReadExt + AsyncWriteExt + AsyncSeekExt + Unpin> RwData for T {}

struct DatabaseData<T: RwData> {
  data: T,
  buffer: Vec<u8>,
}
impl<T: RwData> DatabaseData<T> {
  pub fn new(data: T) -> Self {
    Self {
      data,
      buffer: Vec::with_capacity(8 * 1024),
    }
  }

  pub async fn read(&mut self, region: Range<usize>) -> Result<&mut [u8]> {
    let region_size = region.len();
    self.buffer.clear();
    self.buffer.extend((0..region_size).map(|_| 0));
    self.data.read_exact(&mut self.buffer[..]).await?;
    Ok(&mut self.buffer[..])
  }
}

pub struct Database<T: RwData> {
  data: DatabaseData<T>,
  shape: DatabaseShape,
}

impl<T: RwData> Database<T> {
  pub async fn init(data: T) -> Result<Self> {
    let mut data = DatabaseData::new(data);
    let shape_length = data.read(0..1).await?.len();
    let shape_bytes = &*data.read(1..(shape_length + 1)).await?;
    let shape = rkyv::from_bytes(shape_bytes)?;
    Ok(Self { data, shape })
  }
}
