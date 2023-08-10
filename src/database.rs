use std::{
  io::{Read, Write, Seek, SeekFrom},
  ops::Range,
  iter::repeat,
  fs::File,
};
use anyhow::{Result, ensure};
use divrem::DivCeil;
use crate::{shape::DatabaseShape, types::ReprSize};

//pub const SECTOR_SIZE: usize = 128 * 1024 * 1024;
pub const SECTOR_SIZE: usize = 1024;

pub trait RwData: Read + Write + Seek {}
impl<T: Read + Write + Seek> RwData for T {}

pub struct Database<T: RwData> {
  data: T,
  pub shape: DatabaseShape,
  sector_count: u64,
  shape_dirty: bool,
  shape_location: Range<u64>,
}

impl<T: RwData> Database<T> {
  pub fn new(data: T) -> Result<Self> {
    Ok(Self {
      data,
      shape: DatabaseShape::default(),
      sector_count: 1,
      shape_dirty: false,
      shape_location: 0..0
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
    let shape_start_bytes = self.shape_location.start * SECTOR_SIZE as u64;
    let shape_size_sectors = self.shape_location.end - self.shape_location.start;
    let shape_size_bytes = shape_size_sectors as usize * SECTOR_SIZE;
    let mut buffer = vec![0; shape_size_bytes];
    self.data.seek(SeekFrom::Start(shape_start_bytes))?;
    self.data.read_exact(&mut buffer)?;
    self.shape = bincode::deserialize(&buffer)?;
    Ok(())
  }
  
  pub fn write_shape(&mut self) -> Result<()> {
    let mut shape_size_sectors = self.shape_location.end - self.shape_location.start;
    let mut shape_size_bytes = shape_size_sectors as usize * SECTOR_SIZE;
    
    let mut buffer = bincode::serialize(&self.shape)?;

    if buffer.len() > shape_size_bytes {
      //TODO check if already on the edge
      //buffer too large, we need to move the shape!
      for sec in self.shape_location.clone().rev() {
        self.reclaim_sector(sec);
      }
      let buffer_size_sectors = DivCeil::div_ceil(buffer.len() as u64, SECTOR_SIZE as u64);
      self.shape_location = self.allocate_consecutive_sectors(buffer_size_sectors);
      self.shape_dirty = true;
      self.write_shape_location()?;
      shape_size_sectors = self.shape_location.end - self.shape_location.start;
      shape_size_bytes = shape_size_sectors as usize * SECTOR_SIZE;
      //println!("shape_size_bytes = {shape_size_bytes}\nbuf.len = {}", buffer.len());
    }

    //extend buffer to match sector len
    buffer.extend(repeat(0).take(shape_size_bytes - buffer.len()));
    
    //write sector data
    //not using write_sector because we're writing to multiple sectors at the same time!
    let shape_start_bytes = self.shape_location.start * SECTOR_SIZE as u64;
    self.data.seek(SeekFrom::Start(shape_start_bytes))?;
    self.data.write_all(&buffer)?;

    //no longer dirty!
    self.shape_dirty = false;
    
    Ok(())
  }

  pub fn reclaim_sector(&mut self, sector: u64) {
    // if sector == self.sector_count - 1 {
    //   self.sector_count -= 1;
    //   //TODO reclaim space here
    //   return;
    // }
    self.shape.reclaim.push_back(sector);
    self.shape_dirty = true;
  }

  pub fn allocate_sector(&mut self) -> u64 {
    if let Some(sector) = self.shape.reclaim.pop_front() {
      self.shape_dirty = true;
      sector
    } else {
      self.sector_count += 1;
      self.sector_count - 1
    }
  }

  pub fn allocate_multiple_sectors(&mut self, buf: &mut [u64]) {
    if buf.is_empty() {
      return
    }
    if buf.len() == 1 {
      buf[0] = self.allocate_sector();
    } else {
      self.shape_dirty = true;
      for entry in buf {
        if let Some(sector) = self.shape.reclaim.pop_front() {
          *entry = sector;
          continue
        }
        *entry = self.sector_count;
        self.sector_count += 1;
      }
    }
  }

  pub fn allocate_consecutive_sectors(&mut self, len: u64) -> Range<u64> {
    //TODO: look through reclaim
    self.sector_count += len;
    (self.sector_count - len)..self.sector_count
  }

  pub fn read_shape_location(&mut self) -> Result<()> {
    let mut range_start = [0; 8];
    let mut range_end = [0; 8];
    self.data.seek(SeekFrom::Start(0))?;
    self.data.read_exact(&mut range_start[..])?;
    self.data.read_exact(&mut range_end[..])?;
    let range_start = u64::from_le_bytes(range_start);
    let range_end = u64::from_le_bytes(range_end);
    self.shape_location = range_start..range_end;
    Ok(())
  }

  pub fn write_shape_location(&mut self) -> Result<()> {
    self.data.seek(SeekFrom::Start(0))?;
    self.data.write_all(&u64::to_le_bytes(self.shape_location.start))?;
    self.data.write_all(&u64::to_le_bytes(self.shape_location.end))?;
    Ok(())
  }

  pub fn read_sector_count(&mut self) -> Result<()> {
    self.sector_count = (self.data.seek(SeekFrom::End(0))? / SECTOR_SIZE as u64).max(2);
    Ok(())
  }
  
  pub fn read_database(&mut self) -> Result<()> {
    self.read_sector_count()?;
    self.read_shape_location()?;
    self.read_shape()?;
    println!("sector count: {:?}", self.sector_count);
    println!("shape location: {:?}", self.shape_location);
    println!("shape: {:?}", self.shape);
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
    
    //get offset and sector
    let offset = row_size * (table.row_count as usize - falls_into_fragment * entries_per_fragment);
    let mut sector = table.fragmentation.get(falls_into_fragment).copied();

    //increment row count
    table.row_count += 1;
    
    //fragment table if needed
    if table.fragmentation.len() <= falls_into_fragment {
      let psector = self.allocate_sector();
      //HACK: re-grab table to avoid borrowing issues
      self.shape.tables.get_mut(name).unwrap().fragmentation.push(psector);
      sector = Some(psector);
    }

    //write data
    self.write_shape()?;
    self.write_sector(sector.unwrap(), data, offset)?;

    Ok(())
  }
}
