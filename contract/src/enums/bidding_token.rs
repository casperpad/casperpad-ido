use core::mem;

use alloc::{collections::BTreeMap, vec::Vec};
use casper_types::{
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, ContractHash, U256,
};

#[derive(Debug, Clone)]
pub enum BiddingToken {
    Native {
        price: Option<U256>,
    },
    ERC20s {
        tokens_with_price: BTreeMap<ContractHash, U256>,
    },
}

impl CLTyped for BiddingToken {
    fn cl_type() -> casper_types::CLType {
        CLType::Any
    }
}

impl ToBytes for BiddingToken {
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let mut buffer = bytesrepr::allocate_buffer(self)?;
        match self {
            BiddingToken::Native { price } => {
                buffer.insert(0, 0u8);
                buffer.extend(price.to_bytes()?);
            }
            BiddingToken::ERC20s { tokens_with_price } => {
                buffer.insert(0, 1u8);
                buffer.extend(tokens_with_price.to_bytes()?);
            }
        }
        Ok(buffer)
    }

    fn serialized_length(&self) -> usize {
        mem::size_of::<u8>()
            + match self {
                BiddingToken::Native { price } => price.serialized_length(),
                BiddingToken::ERC20s { tokens_with_price } => tokens_with_price.serialized_length(),
            }
    }

    fn into_bytes(self) -> Result<Vec<u8>, bytesrepr::Error>
    where
        Self: Sized,
    {
        self.to_bytes()
    }
}

impl FromBytes for BiddingToken {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (tag, bytes) = u8::from_bytes(bytes)?;
        match tag {
            0 => {
                let (price, bytes) = Option::<U256>::from_bytes(bytes)?;
                Ok((BiddingToken::Native { price }, bytes))
            }
            1 => {
                let (tokens_with_price, bytes) = BTreeMap::<ContractHash, U256>::from_bytes(bytes)?;
                Ok((BiddingToken::ERC20s { tokens_with_price }, bytes))
            }
            _ => Err(bytesrepr::Error::Formatting),
        }
    }
}
