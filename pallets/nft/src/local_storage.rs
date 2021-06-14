use crate::{
    Config,
    Error,
};
use sp_runtime::{
    offchain::{
		storage::{StorageValueRef},
	},
};
use sp_std::{
    fmt,
    vec::Vec,
};

type KeyType<'a> = &'a [u8];

pub struct VecKey(pub Vec<u8>);

impl fmt::LowerHex for VecKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

pub struct LocalStorageValue<'a, R>{
    pub key: KeyType<'a>,
    pub default_value: R,
}

impl<'a, R: codec::Codec + Default + Clone>LocalStorageValue<'a, R> {
    pub fn new(key: KeyType<'a>) -> Self {
        LocalStorageValue{
            key,
            default_value: Default::default(),
        }
    }

    pub fn get_ref(&self) -> StorageValueRef {
        StorageValueRef::persistent(&self.key)
    }

    pub fn get<C: Config>(&self) -> Result<R, Error<C>> {
        let result = StorageValueRef::persistent(&self.key)
            .get::<R>()
            .ok_or(Error::<C>::OffchainValueNotFound)?
            .ok_or(Error::<C>::OffchainValueDecode)?;

        Ok(result)
	}

    pub fn mutate<C: Config>(
        &self,
        mutate_closure: impl FnOnce(R) -> Result<R, Error<C>>
    ) -> Result<R, Error<C>> {
        let result = self.get_ref().mutate(|x: Option<Option<R>>| {
            match x {
                Some(Some(value)) => {
                    mutate_closure(value)
                },
                _ => {
                    Ok(self.default_value.clone())
                },
            }
        });
    
        match result {
            Ok(Ok(x)) => {
                Ok(x)
            },
            Err(e) => Err(e),
            Ok(Err(_x)) => Err(Error::<C>::OffchainLock),
        }
    }
}