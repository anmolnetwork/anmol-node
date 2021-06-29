use crate::{Config, Error};
use sp_std::vec::Vec;

pub fn remove_vector_item<C: Config, T: Ord>(
	mut vector: Vec<T>,
	item: &T,
) -> Result<Vec<T>, Error<C>> {
	match vector.binary_search(item) {
		Ok(index) => {
			vector.remove(index);
			Ok(vector)
		}
		Err(_) => Err(Error::<C>::RemoveVectorItem),
	}
}
