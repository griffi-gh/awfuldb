use std::ops::Range;
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncSeekExt};
use crate::shape::DatabaseShape;

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

//TODO : how to handle shape growth?

impl<T: RwData> Database<T> {
  pub async fn init(data: T) -> Result<Self> {
    let mut data = DatabaseData::new(data);
    let shape_length = u64::from_le_bytes(data.read(0..8).await?.try_into().unwrap()) as usize;
    let shape_bytes = &*data.read(8..(shape_length + 8)).await?;
    let shape = bincode::deserialize(shape_bytes)?;
    Ok(Self { data, shape })
  }
}
