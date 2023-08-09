use rkyv::{Archive, Serialize, Deserialize};

pub trait ReprSize {
  fn byte_size(&self) -> usize;
}

#[derive(Archive, Serialize, Deserialize, Clone, Copy)]
#[archive(check_bytes)]
#[repr(u8)]
pub enum IntegerSize {
  Int8 = 1,
  Int16 = 2,
  Int32 = 4,
  Int64 = 8,
}

impl ReprSize for IntegerSize {
  fn byte_size(&self) -> usize {
    *self as u8 as usize
  }
}

#[derive(Archive, Serialize, Deserialize, Clone, Copy)]
#[archive(check_bytes)]
#[repr(u8)]
pub enum FloatSize {
  Float32 = 4,
  Float64 = 8,
}

impl ReprSize for FloatSize {
  fn byte_size(&self) -> usize {
    *self as u8 as usize
  }
}

#[derive(Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct IntegerType {
  pub size: IntegerSize,
  pub is_signed: bool,
}

impl ReprSize for IntegerType {
  fn byte_size(&self) -> usize {
    self.size.byte_size()
  }
}

#[derive(Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct FloatType {
  pub size: FloatSize,
}

impl ReprSize for FloatType {
  fn byte_size(&self) -> usize {
    self.size.byte_size()
  }
}

#[derive(Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub enum NumberType {
  Integer(IntegerType),
  Float(FloatType),
}

impl ReprSize for NumberType {
  fn byte_size(&self) -> usize {
    match self {
      NumberType::Integer(i) => i.byte_size(),
      NumberType::Float(f) => f.byte_size(),
    }
  }
}

#[derive(Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct TextType {
  pub size: usize,
}

impl ReprSize for TextType {
  fn byte_size(&self) -> usize {
    self.size
  }
}

#[derive(Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct BlobType {
  pub size: usize,
}

impl ReprSize for BlobType {
  fn byte_size(&self) -> usize {
    self.size
  }
}

#[derive(Archive, Serialize, Deserialize)]
pub enum Type {
  Number(NumberType),
  Text(TextType),
  Blob(BlobType),
  //Time(DateTime<Utc>),
}

impl Type {
  pub fn byte_size(&self) -> usize {
    match self {
      Type::Number(n) => n.byte_size(),
      Type::Text(t) => t.byte_size(),
      Type::Blob(b) => b.byte_size(),
      //Type::Time(t) => todo!(),
    }
  }
}

// impl From<Type> for &'static str {
//   fn from(typ: Type) -> &'static str {
//     impl typ {
      
//     }
//   }
// }
