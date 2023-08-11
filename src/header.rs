use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct DbHeader {
  /// range of consecutive sectors containing the shape
  /// we're not using range directly as it's not `Copy`able
  pub shape_location: (u64, u64),
  pub sector_count: u64,
}

impl Default for DbHeader {
  fn default() -> Self {
    Self {
      shape_location: (0, 0),
      sector_count: 1,
    }
  }
}
