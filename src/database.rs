use std::io::{Read, Write, Seek, SeekFrom};
use anyhow::{Result, ensure};
use crate::{shape::DatabaseShape, types::ReprSize};

//pub const SECTOR_SIZE: usize = 128 * 1024 * 1024;
pub const SECTOR_SIZE: usize = 4096;

pub trait RwData: Read + Write + Seek {}
impl<T: Read + Write + Seek> RwData for T {}

pub struct Database<T: RwData> {
  data: T,
  pub shape: DatabaseShape,
  sector_count: u64,
}

impl<T: RwData> Database<T> {
  pub fn new(data: T) -> Result<Self> {
    Ok(Self {
      data,
      shape: DatabaseShape::default(),
      sector_count: 0,
    })
  }

  pub fn read_sector(&mut self, sector: u64) -> Result<Box<[u8]>> {
    let mut buffer = vec![0; SECTOR_SIZE].into_boxed_slice();
    self.data.seek(SeekFrom::Start(sector * SECTOR_SIZE as u64))?;
    self.data.read_exact(&mut buffer[..])?;
    Ok(buffer)
  }

  pub fn write_sector(&mut self, sector: u64, data: &[u8], offset: usize) -> Result<()> {
    ensure!(data.len() <= SECTOR_SIZE);
    ensure!((data.len() + offset) <= SECTOR_SIZE);

    //write data
    self.data.seek(SeekFrom::Start(offset as u64 + sector * SECTOR_SIZE as u64))?;
    self.data.write_all(data)?;

    //if writing a non-sector-sized buffer a new sector...
    //...seek to the last byte and write something to ensure valid file size
    if ((data.len() + offset) < SECTOR_SIZE) && (sector >= self.sector_count) {
      self.data.seek(SeekFrom::Start((sector + 1) * SECTOR_SIZE as u64 - 1))?;
      self.data.write_all(&[0])?;
    }

    //update sector count
    self.sector_count = self.sector_count.max(sector + 1);
    
    Ok(())
  }

  pub fn read_shape(&mut self) -> Result<()> {
    let buffer = self.read_sector(0)?;
    self.shape = bincode::deserialize(&buffer)?;
    Ok(())
  }
  
  pub fn write_shape(&mut self) -> Result<()> {
    let buffer = bincode::serialize(&self.shape)?;
    self.write_sector(0, &buffer, 0)?;
    Ok(())
  }

  pub fn read_sector_count(&mut self) -> Result<()> {
    self.sector_count = (self.data.seek(SeekFrom::End(0))? / SECTOR_SIZE as u64).max(1);
    Ok(())
  }
  
  pub fn read_database(&mut self) -> Result<()> {
    self.read_sector_count()?;
    self.read_shape()?;
    Ok(())
  }

  //TODO: proper error handling (error enum)
  //TODO: ensure that table exists
  //TODO: accept sth like Row instead of raw bytes

  pub fn table_insert(&mut self, name: &str, data: &[u8]) -> Result<()> {
    let table = self.shape.tables.get_mut(name).unwrap();
    
    let row_size = table.byte_size();
    
    let entries_per_fragment = SECTOR_SIZE / row_size;
    let falls_into_fragment = table.row_count as usize / entries_per_fragment;
    
    //ensure data size
    ensure!(row_size == data.len());
    
    //fragment table if needed
    if table.fragmentation.len() <= falls_into_fragment {
      table.fragmentation.push(self.sector_count);
    }
    
    //get sector and offset
    let sector = table.fragmentation[falls_into_fragment];
    let offset = row_size * (table.row_count as usize - falls_into_fragment * entries_per_fragment);
    
    //increment row count
    table.row_count += 1;
    
    //write data
    self.write_shape()?;
    self.write_sector(sector, data, offset)?;

    Ok(())
  }
}
