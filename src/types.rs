use serde::{Serialize, Deserialize};

pub trait ReprSize {
  fn byte_size(&self) -> usize;
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct IntegerType {
  pub size: IntegerSize,
  pub is_signed: bool,
}

impl ReprSize for IntegerType {
  fn byte_size(&self) -> usize {
    self.size.byte_size()
  }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct FloatType {
  pub size: FloatSize,
}

impl ReprSize for FloatType {
  fn byte_size(&self) -> usize {
    self.size.byte_size()
  }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct TextType {
  pub size: usize,
}

impl ReprSize for TextType {
  fn byte_size(&self) -> usize {
    self.size
  }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct BlobType {
  pub size: usize,
}

impl ReprSize for BlobType {
  fn byte_size(&self) -> usize {
    self.size
  }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum TypeTree {
  Number(NumberType),
  Text(TextType),
  Blob(BlobType),
  //TODO Time
}

impl TypeTree {
  pub fn byte_size(&self) -> usize {
    match self {
      TypeTree::Number(n) => n.byte_size(),
      TypeTree::Text(t) => t.byte_size(),
      TypeTree::Blob(b) => b.byte_size(),
    }
  }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Type {
  Unsigned8,
  Unsigned16,
  Unsigned32,
  Unsigned64,
  Signed8,
  Signed16,
  Signed32,
  Signed64,
  Float32,
  Float64,
  Text(usize),
  Blob(usize),
}

impl Type {
  pub const fn from_type_tree(tree: TypeTree) -> Self {
    match tree {
      TypeTree::Number(n) => match n {
        NumberType::Integer(i) => match i.size {
          IntegerSize::Int8 => if i.is_signed { Type::Signed8 } else { Type::Unsigned8 },
          IntegerSize::Int16 => if i.is_signed { Type::Signed16 } else { Type::Unsigned16 },
          IntegerSize::Int32 => if i.is_signed { Type::Signed32 } else { Type::Unsigned32 },
          IntegerSize::Int64 => if i.is_signed { Type::Signed64 } else { Type::Unsigned64 },
        },
        NumberType::Float(f) => match f.size {
          FloatSize::Float32 => Type::Float32,
          FloatSize::Float64 => Type::Float64,
        }
      },
      TypeTree::Blob(b) => Type::Blob(b.size),
      TypeTree::Text(t) => Type::Text(t.size),
    }
  }

  pub const fn into_type_tree(self) -> TypeTree {
    match self {
      Type::Unsigned8 => TypeTree::Number(NumberType::Integer(IntegerType { size: IntegerSize::Int8, is_signed: false })),
      Type::Unsigned16 => TypeTree::Number(NumberType::Integer(IntegerType { size: IntegerSize::Int16, is_signed: false })),
      Type::Unsigned32 => TypeTree::Number(NumberType::Integer(IntegerType { size: IntegerSize::Int32, is_signed: false })),
      Type::Unsigned64 => TypeTree::Number(NumberType::Integer(IntegerType { size: IntegerSize::Int64, is_signed: false })),
      Type::Signed8 => TypeTree::Number(NumberType::Integer(IntegerType { size: IntegerSize::Int8, is_signed: true })),
      Type::Signed16 => TypeTree::Number(NumberType::Integer(IntegerType { size: IntegerSize::Int16, is_signed: true })),
      Type::Signed32 => TypeTree::Number(NumberType::Integer(IntegerType { size: IntegerSize::Int32, is_signed: true })),
      Type::Signed64 => TypeTree::Number(NumberType::Integer(IntegerType { size: IntegerSize::Int64, is_signed: true })),
      Type::Float32 => TypeTree::Number(NumberType::Float(FloatType { size: FloatSize::Float32 })),
      Type::Float64 => TypeTree::Number(NumberType::Float(FloatType { size: FloatSize::Float64 })),
      Type::Text(size) => TypeTree::Text(TextType { size }),
      Type::Blob(size) => TypeTree::Blob(BlobType { size }),
    }
  }
}

impl TypeTree {
  pub const fn from_type(value: Type) -> Self {
    value.into_type_tree()
  }

  pub const fn into_type(self) -> Type {
    Type::from_type_tree(self)
  }
}

impl From<Type> for TypeTree {
  fn from(value: Type) -> Self {
    value.into_type_tree()
  }
}

impl From<TypeTree> for Type {
  fn from(tree: TypeTree) -> Self {
    Type::from_type_tree(tree)
  }
}
