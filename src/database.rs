use std::{
  io::{Read, Write, Seek, SeekFrom},
  ops::Range,
  iter::repeat,
  fs::File,
};
use anyhow::{Result, ensure};
use divrem::DivCeil;
use crate::{shape::DbShape, types::ReprSize, header::DbHeader};

//pub const SECTOR_SIZE: usize = 128 * 1024 * 1024;
pub const SECTOR_SIZE: usize = 1024;

pub trait RwData: Read + Write + Seek {}
impl<T: Read + Write + Seek> RwData for T {}

pub struct Database<T: RwData> {
  data: T,
  pub header: DbHeader,
  pub shape: DbShape,
  header_dirty: bool,
  shape_dirty: bool,
}

impl<T: RwData> Database<T> {
  pub fn new(data: T) -> Result<Self> {
    Ok(Self {
      data,
      header: DbHeader::default(),
      shape: DbShape::default(),
      header_dirty: true,
      shape_dirty: true,
    })
  }

  pub(crate) fn mark_shape_dirty(&mut self) {
    self.shape_dirty = true;
  }

  pub(crate) fn mark_header_dirty(&mut self) {
    self.header_dirty = true;
  }

  pub fn read_sector(&mut self, sector: u64) -> Result<Box<[u8]>> {
    let mut buffer = vec![0; SECTOR_SIZE].into_boxed_slice();
    self.data.seek(SeekFrom::Start(sector * SECTOR_SIZE as u64))?;
    self.data.read_exact(&mut buffer[..])?;
    Ok(buffer)
  }

  pub fn write_sector(&mut self, sector: u64, data: &[u8], offset: usize) -> Result<()> {
    ensure!(sector < self.header.sector_count, "Unallocated sector");
    ensure!((data.len() + offset) <= SECTOR_SIZE, "Data does not fit inside the sector");

    //write data
    self.data.seek(SeekFrom::Start(offset as u64 + sector * SECTOR_SIZE as u64))?;
    self.data.write_all(data)?;

    //if writing a non-sector-sized buffer a new sector...
    //...seek to the last byte and write something to ensure valid file size
    if ((data.len() + offset) < SECTOR_SIZE) && (sector >= self.header.sector_count) {
      self.data.seek(SeekFrom::Start((sector + 1) * SECTOR_SIZE as u64 - 1))?;
      self.data.write_all(&[0])?;
    }

    //update sector count
    //XXX: we're using allocation api now so no need for this anymore
    // self.header.sector_count = self.header.sector_count.max(sector + 1);
    // self.header_dirty = true;
    
    Ok(())
  }

  pub fn read_header(&mut self) -> Result<()> {
    let buf = self.read_sector(0)?;
    self.header = bincode::deserialize(&buf)?;
    self.header_dirty = false;
    Ok(())
  }

  pub fn write_header(&mut self) -> Result<()> {
    let mut buf = vec![0; SECTOR_SIZE].into_boxed_slice();
    bincode::serialize_into(&mut buf[..], &self.header)?;
    self.write_sector(0, &buf, 0)?;
    self.header_dirty = false;
    Ok(())
  }

  pub fn read_shape(&mut self) -> Result<()> {
    let shape_start_bytes = self.header.shape_location.0 * SECTOR_SIZE as u64;
    let shape_size_sectors = self.header.shape_location.1 - self.header.shape_location.0;
    let shape_size_bytes = shape_size_sectors as usize * SECTOR_SIZE;
    let mut buffer = vec![0; shape_size_bytes];
    self.data.seek(SeekFrom::Start(shape_start_bytes))?;
    self.data.read_exact(&mut buffer)?;
    self.shape = bincode::deserialize(&buffer)?;
    self.shape_dirty = false;
    Ok(())
  }
  
  pub fn write_shape(&mut self) -> Result<()> {
    let mut shape_size_sectors = self.header.shape_location.1 - self.header.shape_location.0;
    let mut shape_size_bytes = shape_size_sectors as usize * SECTOR_SIZE;
    
    let mut buffer = bincode::serialize(&self.shape)?;

    if buffer.len() > shape_size_bytes {
      //TODO check if already on the edge
      //buffer too large, we need to move the shape!
      //Mark shape as NOT dirty NOW so we can watch for changes
      self.shape_dirty = false;
      for sec in (self.header.shape_location.0..self.header.shape_location.1).rev() {
        self.reclaim_sector(sec);
      }
      if self.shape_dirty {
        //Re-serialize because shape changed
        buffer = bincode::serialize(&self.shape)?;
      }
      let buffer_size_sectors = DivCeil::div_ceil(buffer.len() as u64, SECTOR_SIZE as u64);
      let alloc_sec_range = self.allocate_consecutive_sectors(buffer_size_sectors);
      self.header.shape_location = (alloc_sec_range.start, alloc_sec_range.end);
      self.header_dirty = true;
      shape_size_sectors = self.header.shape_location.1 - self.header.shape_location.0;
      shape_size_bytes = shape_size_sectors as usize * SECTOR_SIZE;
      //println!("shape_size_bytes = {shape_size_bytes}\nbuf.len = {}", buffer.len());
    }

    //extend buffer to match sector len
    buffer.extend(repeat(0).take(shape_size_bytes - buffer.len()));
    
    //write sector data
    //not using write_sector because we're writing to multiple sectors at the same time!
    let shape_start_bytes = self.header.shape_location.0 * SECTOR_SIZE as u64;
    self.data.seek(SeekFrom::Start(shape_start_bytes))?;
    self.data.write_all(&buffer)?;

    //no longer dirty!
    self.shape_dirty = false;
    
    Ok(())
  }

  pub fn reclaim_sector(&mut self, sector: u64) {
    if sector == self.header.sector_count - 1 {
      self.header.sector_count -= 1;
      self.header_dirty = true;
    } else {
      self.shape.reclaim.push_back(sector);
      self.shape_dirty = true;
    }
  }

  pub fn allocate_sector(&mut self) -> u64 {
    if let Some(sector) = self.shape.reclaim.pop_front() {
      self.shape_dirty = true;
      sector
    } else {
      self.header_dirty = true;
      self.header.sector_count += 1;
      self.header.sector_count - 1
    }
  }

  pub fn allocate_multiple_sectors(&mut self, buf: &mut [u64]) {
    if buf.is_empty() {
      return
    }
    if buf.len() == 1 {
      buf[0] = self.allocate_sector();
    } else {
      for entry in buf {
        if let Some(sector) = self.shape.reclaim.pop_front() {
          *entry = sector;
          self.shape_dirty = true;
          continue
        }
        *entry = self.header.sector_count;
        self.header_dirty = true;
        self.header.sector_count += 1;
      }
    }
  }

  pub fn allocate_consecutive_sectors(&mut self, len: u64) -> Range<u64> {
    if len == 0 {
      0..0
    } else if len == 1 {
      let sec = self.allocate_sector();
      sec..(sec + 1)
    } else {
      self.header_dirty = true;
      self.header.sector_count += len;
      (self.header.sector_count - len)..self.header.sector_count
    }
  }
  
  /// Read shape and header from the drive\
  /// This is not called automatically!\
  /// You need to call it explicitly to prevent data loss\
  pub fn read_database(&mut self) -> Result<()> {
    //Order of operations is important here!
    //Reading the shape requires shape location to be known which is located in the header
    self.read_header()?;
    self.read_shape()?;
    println!("header: {:?}", self.header);
    println!("shape: {:?}", self.shape);
    Ok(())
  }

  /// Sync modified header/shape data to the disk\
  /// This is not called automatically!\
  /// You need to call it explicitly to prevent data loss\
  pub fn sync_database(&mut self) -> Result<()> {
    //Order of operations is important here too!
    //As writing the shape may cause shape to be relocated (which modifies the header)
    if self.shape_dirty {
      self.write_shape()?;
    }
    if self.header_dirty {
      self.write_header()?;
    }
    Ok(())
  }

  /// Defragment and optimize the database\
  /// Currently a no-op
  pub fn optimize(&mut self) -> Result<()> {
    //TODO database optimization
    Ok(())
  }

  //TODO: proper error handling (error enum)
  //TODO: ensure that table exists
  //TODO: accept sth like Row instead of raw bytes

  /// Warning: db data is reflected right away, but shape is not
  /// Remember to call `sync_to_disk` to write that info to the disk
  pub fn table_insert(&mut self, name: &str, data: &[u8]) -> Result<()> {
    let table = self.shape.get_table_mut(name).unwrap();

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
      self.shape.get_table_mut(name).unwrap().fragmentation.push(psector);
      sector = Some(psector);
    }

    //write data
    self.write_sector(sector.unwrap(), data, offset)?;

    //mark shape as dirty
    self.shape_dirty = true;

    Ok(())
  }

  pub fn table_read_row_column(&mut self, name: &str, row: u64, column: usize) -> Result<Box<[u8]>> {
    let table = self.shape.get_table(name).unwrap();
    let row_size = table.byte_size();
    let entries_per_fragment = SECTOR_SIZE / row_size;
    let falls_into_fragment = row / entries_per_fragment as u64;
    let sector = table.fragmentation[falls_into_fragment as usize];
    let mut buffer = vec![0; table.columns[column].typ.into_type_tree().byte_size()].into_boxed_slice();
    let row_offset = row_size * (row as usize - falls_into_fragment as usize * entries_per_fragment);
    let col_offset: usize = table.columns[..column]
      .iter()
      .map(|col| col.typ.into_type_tree().byte_size())
      .sum();
    self.data.seek(SeekFrom::Start(sector * SECTOR_SIZE as u64 + col_offset as u64 + row_offset as u64))?;
    self.data.read_exact(&mut buffer[..])?;
    Ok(buffer)
  }
}

impl Database<File> {
  /// Commit in-memory data to the filesystem\
  /// This is not called automatically!\
  /// This function DOES NOT call `sync_database`
  pub fn sync_fs(&mut self) -> Result<()> {
    self.data.sync_all()?;
    Ok(())
  }

  /// Truncate the database file size based on the current sector count\
  /// This is not called automatically!\
  pub fn truncate(&mut self) -> Result<()> {
    let sector_len = self.header.sector_count * SECTOR_SIZE as u64;
    let data_len = self.data.metadata()?.len();
    if data_len > sector_len {
      self.data.set_len(sector_len)?;
    }
    Ok(())
  }
}
