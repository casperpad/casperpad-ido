use casper_types::{CLType,CLTyped, Key, bytesrepr::ToBytes,bytesrepr::{self, FromBytes}};

struct Project {
  id:String,
}

// The struct `Project` can me treated as CLType
impl CLTyped for Project {
  fn cl_type() -> CLType {
      CLType::ByteArray(10u32)
  }
}

// Serialize for Project
impl ToBytes for Project {
  #[inline(always)]
  fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error>{
      let bytes = self.id.as_bytes();
      Ok(bytes.to_vec())
  }

  #[inline(always)]
  fn serialized_length(&self) -> usize {
      self.id.serialized_length()
  }

  fn into_bytes(self) -> Result<Vec<u8>, casper_types::bytesrepr::Error>
  where
      Self: Sized,
  {
      self.to_bytes()
  }
}

// Deserialize for Project
impl FromBytes for Project {
  fn from_bytes(bytes: &[u8]) -> Result<(Self,&[u8]), bytesrepr::Error> {
      let (string,remainder) = String::from_bytes(bytes).unwrap();
      let project = Project::new(string.as_str());
      Ok((project,remainder))
  }
}

impl Project{
  fn new(id:&str)->Self{
      Self {
          id: String::from(id),
      }
  }
}