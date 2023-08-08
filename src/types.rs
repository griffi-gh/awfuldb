use rkyv::Archive;

pub trait ReprSize {
  fn byte_size(&self) -> usize;
}

#[derive(Archive)]
pub enum IntegerSize {
  Int8,
  Int16,
  Int32,
  Int64,
}

impl ReprSize for IntegerSize {
  fn byte_size(&self) -> usize {
    match self {
      Self::Int8 => 1,
      Self::Int16 => 2,
      Self::Int32 => 4,
      Self::Int64 => 8,
    }
  }
}

#[derive(Archive)]
pub enum FloatSize {
  Float32,
  Float64,
}

impl ReprSize for FloatSize {
  fn byte_size(&self) -> usize {
    match self {
      Self::Float32 => 4,
      Self::Float64 => 8,
    }
  }
}

#[derive(Archive)]
pub struct IntegerType {
  pub size: IntegerSize,
  pub is_signed: bool,
}

impl ReprSize for IntegerType {
  fn byte_size(&self) -> usize {
    self.size.byte_size()
  }
}

#[derive(Archive)]
pub struct FloatType {
  pub size: FloatSize,
}

impl ReprSize for FloatType {
  fn byte_size(&self) -> usize {
    self.size.byte_size()
  }
}

#[derive(Archive)]
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

#[derive(Archive)]
pub struct TextType {
  pub size: usize,
}

impl ReprSize for TextType {
  fn byte_size(&self) -> usize {
    self.size
  }
}

#[derive(Archive)]
pub struct BlobType {
  pub size: usize,
}

impl ReprSize for BlobType {
  fn byte_size(&self) -> usize {
    self.size
  }
}

#[derive(Archive)]
pub enum Type {
  Number(NumberType),
  Text(TextType),
  Blob(BlobType),
}

impl Type {
  pub fn byte_size(&self) -> usize {
    match self {
      Type::Number(n) => n.byte_size(),
      Type::Text(t) => t.byte_size(),
      Type::Blob(b) => b.byte_size(),
    }
  }
}

// impl From<Type> for &'static str {
//   fn from(typ: Type) -> &'static str {
//     impl typ {
      
//     }
//   }
// }
