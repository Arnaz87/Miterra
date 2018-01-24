
pub trait VoxelSource {
  fn get(&self, x: i32, y: i32, z: i32) -> bool;
}

pub struct SphereSource {
  pub x: i32,
  pub y: i32,
  pub z: i32,
  pub r: i32,
}

impl VoxelSource for SphereSource {
  fn get(&self, ix: i32, iy: i32, iz: i32) -> bool {
    let (x, y, z) = (ix-self.x, iy-self.y, iz-self.z);
    let d2 = x*x + y*y + z*z;
    let r2 = self.r * self.r;
    return d2 < r2;
  }
}