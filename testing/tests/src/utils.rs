use casper_types::{
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped,
};

#[derive(Debug, Clone)]
pub struct Schedule {
    pub unlock_time: i64,
    pub unlock_percent: i64,
}

impl CLTyped for Schedule {
    fn cl_type() -> CLType {
        CLType::ByteArray(16)
    }
}

impl ToBytes for Schedule {
    #[inline(always)]
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut preimage = Vec::new();
        preimage.append(&mut self.unlock_time.to_bytes().unwrap());
        preimage.append(&mut self.unlock_percent.to_bytes().unwrap());
        Ok(preimage)
        // Ok((*self as u32).into_bytes().unwrap().to_vec())
    }

    fn into_bytes(self) -> Result<Vec<u8>, casper_types::bytesrepr::Error>
    where
        Self: Sized,
    {
        self.to_bytes()
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        128
    }
}

impl FromBytes for Schedule {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (unlock_time, remainder1) = i64::from_bytes(bytes).unwrap();
        let (unlock_percent, remainder2) = i64::from_bytes(remainder1).unwrap();
        let schedule = Schedule {
            unlock_time,
            unlock_percent,
        };
        Ok((schedule, remainder2))
    }
}
