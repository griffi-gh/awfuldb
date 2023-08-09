use std::io::{Read, Write, Seek, SeekFrom};
use anyhow::{Result, ensure};
use crate::shape::DatabaseShape;

pub const SECTOR_SIZE: usize = 128 * 1024 * 1024;

pub trait RwData: Read + Write + Seek {}
impl<T: Read + Write + Seek> RwData for T {}

pub struct Database<T: RwData> {
  data: T,
  shape: DatabaseShape,
}

impl<T: RwData> Database<T> {
  pub fn new(data: T) -> Result<Self> {
    Ok(Self {
      data,
      shape: DatabaseShape::default(),
    })
  }

  pub fn read_sector(&mut self, sector: u64) -> Result<Box<[u8]>> {
    let mut buffer = vec![0; SECTOR_SIZE].into_boxed_slice();
    self.data.seek(SeekFrom::Start(sector * SECTOR_SIZE as u64))?;
    self.data.read_exact(&mut buffer)?;
    Ok(buffer)
  }

  pub fn write_sector(&mut self, sector: u64, data: &[u8]) -> Result<()> {
    ensure!(data.len() <= SECTOR_SIZE);

    //write data
    self.data.seek(SeekFrom::Start(sector * SECTOR_SIZE as u64))?;
    self.data.write_all(data)?;

    //if writing a non-sector-sized buffer to the last sector...
    //...seek to the last byte and write something to ensure valid file size
    if data.len() < SECTOR_SIZE {
      let end = self.data.seek(SeekFrom::End(0))?;
      if (end / SECTOR_SIZE as u64) == sector {
        self.data.seek(SeekFrom::End((SECTOR_SIZE - data.len() - 1) as i64))?;
        self.data.write_all(&[0])?;
      }
    }
    
    Ok(())
  }

  pub fn read_shape(&mut self) -> Result<()> {
    let buffer = self.read_sector(0)?;
    self.shape = bincode::deserialize(&buffer)?;
    Ok(())
  }
  
  pub fn write_shape(&mut self) -> Result<()> {
    let buffer = bincode::serialize(&self.shape)?;
    self.write_sector(0, &buffer)?;
    Ok(())
  }
}
