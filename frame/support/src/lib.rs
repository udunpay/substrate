// This file is part of Substrate.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Support code for the runtime.

#![cfg_attr(not(feature = "std"), no_std)]

/// Export ourself as `frame_support` to make tests happy.
extern crate self as frame_support;

#[macro_use]
extern crate bitmask;

#[doc(hidden)]
pub use sp_tracing;

#[cfg(feature = "std")]
pub use serde;
pub use sp_core::Void;
#[doc(hidden)]
pub use sp_std;
#[doc(hidden)]
pub use codec;
#[cfg(feature = "std")]
#[doc(hidden)]
pub use once_cell;
#[doc(hidden)]
pub use paste;
#[cfg(feature = "std")]
#[doc(hidden)]
pub use sp_state_machine::BasicExternalities;
#[doc(hidden)]
pub use sp_io::{storage::root as storage_root, self};
#[doc(hidden)]
pub use sp_runtime::RuntimeDebug;

#[macro_use]
pub mod debug;
#[macro_use]
mod origin;
#[macro_use]
pub mod dispatch;
pub mod storage;
mod hash;
#[macro_use]
pub mod event;
#[macro_use]
pub mod metadata;
#[macro_use]
pub mod inherent;
#[macro_use]
pub mod unsigned;
#[macro_use]
pub mod error;
pub mod traits;
pub mod weights;

pub use self::hash::{
	Twox256, Twox128, Blake2_256, Blake2_128, Identity, Twox64Concat, Blake2_128Concat, Hashable,
	StorageHasher, ReversibleStorageHasher
};
pub use self::storage::{
	StorageValue, StorageMap, StorageDoubleMap, StoragePrefixedMap, IterableStorageMap,
	IterableStorageDoubleMap, migration
};
pub use self::dispatch::{Parameter, Callable, IsSubType};
pub use sp_runtime::{self, ConsensusEngineId, print, traits::Printable};

/// A type that cannot be instantiated.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Never {}

/// Create new implementations of the [`Get`](crate::traits::Get) trait.
///
/// The so-called parameter type can be created in three different ways:
///
/// - Using `const` to create a parameter type that provides a `const` getter.
///   It is required that the `value` is const.
///
/// - Declare the parameter type without `const` to have more freedom when creating the value.
///
/// - Using `storage` to create a storage parameter type. This type is special as it tries to
///   load the value from the storage under a fixed key. If the value could not be found in the
///   storage, the given default value will be returned. It is required that the value implements
///   [`Encode`](codec::Encode) and [`Decode`](codec::Decode). The key for looking up the value
///   in the storage is built using the following formular:
///
///   `twox_128(":" ++ NAME ++ ":")` where `NAME` is the name that is passed as type name.
///
/// # Examples
///
/// ```
/// # use frame_support::traits::Get;
/// # use frame_support::parameter_types;
/// // This function cannot be used in a const context.
/// fn non_const_expression() -> u64 { 99 }
///
/// const FIXED_VALUE: u64 = 10;
/// parameter_types! {
///    pub const Argument: u64 = 42 + FIXED_VALUE;
///    /// Visibility of the type is optional
///    OtherArgument: u64 = non_const_expression();
///    pub storage StorageArgument: u64 = 5;
/// }
///
/// trait Config {
///    type Parameter: Get<u64>;
///    type OtherParameter: Get<u64>;
///    type StorageParameter: Get<u64>;
/// }
///
/// struct Runtime;
/// impl Config for Runtime {
///    type Parameter = Argument;
///    type OtherParameter = OtherArgument;
///    type StorageParameter = StorageArgument;
/// }
/// ```
///
/// # Invalid example:
///
/// ```compile_fail
/// # use frame_support::traits::Get;
/// # use frame_support::parameter_types;
/// // This function cannot be used in a const context.
/// fn non_const_expression() -> u64 { 99 }
///
/// parameter_types! {
///    pub const Argument: u64 = non_const_expression();
/// }
/// ```

#[macro_export]
macro_rules! parameter_types {
	(
		$( #[ $attr:meta ] )*
		$vis:vis const $name:ident: $type:ty = $value:expr;
		$( $rest:tt )*
	) => (
		$( #[ $attr ] )*
		$vis struct $name;
		$crate::parameter_types!(IMPL_CONST $name , $type , $value);
		$crate::parameter_types!( $( $rest )* );
	);
	(
		$( #[ $attr:meta ] )*
		$vis:vis $name:ident: $type:ty = $value:expr;
		$( $rest:tt )*
	) => (
		$( #[ $attr ] )*
		$vis struct $name;
		$crate::parameter_types!(IMPL $name, $type, $value);
		$crate::parameter_types!( $( $rest )* );
	);
	(
		$( #[ $attr:meta ] )*
		$vis:vis storage $name:ident: $type:ty = $value:expr;
		$( $rest:tt )*
	) => (
		$( #[ $attr ] )*
		$vis struct $name;
		$crate::parameter_types!(IMPL_STORAGE $name, $type, $value);
		$crate::parameter_types!( $( $rest )* );
	);
	() => ();
	(IMPL_CONST $name:ident, $type:ty, $value:expr) => {
		impl $name {
			/// Returns the value of this parameter type.
			pub const fn get() -> $type {
				$value
			}
		}

		impl<I: From<$type>> $crate::traits::Get<I> for $name {
			fn get() -> I {
				I::from($value)
			}
		}
	};
	(IMPL $name:ident, $type:ty, $value:expr) => {
		impl $name {
			/// Returns the value of this parameter type.
			pub fn get() -> $type {
				$value
			}
		}

		impl<I: From<$type>> $crate::traits::Get<I> for $name {
			fn get() -> I {
				I::from($value)
			}
		}
	};
	(IMPL_STORAGE $name:ident, $type:ty, $value:expr) => {
		impl $name {
			/// Returns the key for this parameter type.
			pub fn key() -> [u8; 16] {
				$crate::sp_io::hashing::twox_128(
					concat!(":", stringify!($name), ":").as_bytes()
				)
			}

			/// Set the value of this parameter type in the storage.
			///
			/// This needs to be executed in an externalities provided
			/// environment.
			pub fn set(value: &$type) {
				$crate::storage::unhashed::put(&Self::key(), value);
			}

			/// Returns the value of this parameter type.
			///
			/// This needs to be executed in an externalities provided
			/// environment.
			pub fn get() -> $type {
				$crate::storage::unhashed::get(&Self::key()).unwrap_or_else(|| $value)
			}
		}

		impl<I: From<$type>> $crate::traits::Get<I> for $name {
			fn get() -> I {
				I::from(Self::get())
			}
		}
	}
}

/// Macro for easily creating a new implementation of both the `Get` and `Contains` traits. Use
/// exactly as with `parameter_types`, only the type must be `Ord`.
#[macro_export]
macro_rules! ord_parameter_types {
	(
		$( #[ $attr:meta ] )*
		$vis:vis const $name:ident: $type:ty = $value:expr;
		$( $rest:tt )*
	) => (
		$( #[ $attr ] )*
		$vis struct $name;
		$crate::parameter_types!{IMPL $name , $type , $value}
		$crate::ord_parameter_types!{IMPL $name , $type , $value}
		$crate::ord_parameter_types!{ $( $rest )* }
	);
	() => ();
	(IMPL $name:ident , $type:ty , $value:expr) => {
		impl $crate::traits::Contains<$type> for $name {
			fn contains(t: &$type) -> bool { &$value == t }
			fn sorted_members() -> $crate::sp_std::prelude::Vec<$type> { vec![$value] }
			fn count() -> usize { 1 }
			#[cfg(feature = "runtime-benchmarks")]
			fn add(_: &$type) {}
		}
	}
}

#[doc(inline)]
pub use frame_support_procedural::{
	decl_storage, construct_runtime, DebugNoBound, DebugStripped, CloneNoBound, EqNoBound,
	PartialEqNoBound, transactional,
};

/// Return Err of the expression: `return Err($expression);`.
///
/// Used as `fail!(expression)`.
#[macro_export]
macro_rules! fail {
	( $y:expr ) => {{
		return Err($y.into());
	}}
}

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
#[macro_export]
macro_rules! ensure {
	( $x:expr, $y:expr $(,)? ) => {{
		if !$x {
			$crate::fail!($y);
		}
	}}
}

/// Evaluate an expression, assert it returns an expected `Err` value and that
/// runtime storage has not been mutated (i.e. expression is a no-operation).
///
/// Used as `assert_noop(expression_to_assert, expected_error_expression)`.
#[macro_export]
#[cfg(feature = "std")]
macro_rules! assert_noop {
	(
		$x:expr,
		$y:expr $(,)?
	) => {
		let h = $crate::storage_root();
		$crate::assert_err!($x, $y);
		assert_eq!(h, $crate::storage_root());
	}
}

/// Assert an expression returns an error specified.
///
/// Used as `assert_err!(expression_to_assert, expected_error_expression)`
#[macro_export]
#[cfg(feature = "std")]
macro_rules! assert_err {
	( $x:expr , $y:expr $(,)? ) => {
		assert_eq!($x, Err($y.into()));
	}
}

/// Assert an expression returns an error specified.
///
/// This can be used on`DispatchResultWithPostInfo` when the post info should
/// be ignored.
#[macro_export]
#[cfg(feature = "std")]
macro_rules! assert_err_ignore_postinfo {
	( $x:expr , $y:expr $(,)? ) => {
		$crate::assert_err!($x.map(|_| ()).map_err(|e| e.error), $y);
	}
}

/// Assert an expression returns error with the given weight.
#[macro_export]
#[cfg(feature = "std")]
macro_rules! assert_err_with_weight {
	($call:expr, $err:expr, $weight:expr $(,)? ) => {
		if let Err(dispatch_err_with_post) = $call {
			$crate::assert_err!($call.map(|_| ()).map_err(|e| e.error), $err);
			assert_eq!(dispatch_err_with_post.post_info.actual_weight, $weight.into());
		} else {
			panic!("expected Err(_), got Ok(_).")
		}
	}
}

/// Panic if an expression doesn't evaluate to `Ok`.
///
/// Used as `assert_ok!(expression_to_assert, expected_ok_expression)`,
/// or `assert_ok!(expression_to_assert)` which would assert against `Ok(())`.
#[macro_export]
#[cfg(feature = "std")]
macro_rules! assert_ok {
	( $x:expr $(,)? ) => {
		let is = $x;
		match is {
			Ok(_) => (),
			_ => assert!(false, "Expected Ok(_). Got {:#?}", is),
		}
	};
	( $x:expr, $y:expr $(,)? ) => {
		assert_eq!($x, Ok($y));
	}
}

#[cfg(feature = "std")]
#[doc(hidden)]
pub use serde::{Serialize, Deserialize};

#[cfg(test)]
mod tests {
	use super::*;
	use codec::{Codec, EncodeLike};
	use frame_metadata::{
		DecodeDifferent, StorageEntryMetadata, StorageMetadata, StorageEntryType,
		StorageEntryModifier, DefaultByteGetter, StorageHasher,
	};
	use sp_std::{marker::PhantomData, result};
	use sp_io::TestExternalities;

	pub trait Trait {
		type BlockNumber: Codec + EncodeLike + Default;
		type Origin;
	}

	mod module {
		#![allow(dead_code)]

		use super::Trait;

		decl_module! {
			pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
		}
	}
	use self::module::Module;

	decl_storage! {
		trait Store for Module<T: Trait> as Test {
			pub Data get(fn data) build(|_| vec![(15u32, 42u64)]):
				map hasher(twox_64_concat) u32 => u64;
			pub OptionLinkedMap: map hasher(blake2_128_concat) u32 => Option<u32>;
			pub GenericData get(fn generic_data):
				map hasher(identity) T::BlockNumber => T::BlockNumber;
			pub GenericData2 get(fn generic_data2):
				map hasher(blake2_128_concat) T::BlockNumber => Option<T::BlockNumber>;
			pub DataDM config(test_config) build(|_| vec![(15u32, 16u32, 42u64)]):
				double_map hasher(twox_64_concat) u32, hasher(blake2_128_concat) u32 => u64;
			pub GenericDataDM:
				double_map hasher(blake2_128_concat) T::BlockNumber, hasher(identity) T::BlockNumber
				=> T::BlockNumber;
			pub GenericData2DM:
				double_map hasher(blake2_128_concat) T::BlockNumber, hasher(twox_64_concat) T::BlockNumber
				=> Option<T::BlockNumber>;
			pub AppendableDM:
				double_map hasher(blake2_128_concat) u32, hasher(blake2_128_concat) T::BlockNumber => Vec<u32>;
		}
	}

	struct Test;
	impl Trait for Test {
		type BlockNumber = u32;
		type Origin = u32;
	}

	fn new_test_ext() -> TestExternalities {
		GenesisConfig::default().build_storage().unwrap().into()
	}

	type Map = Data;

	trait Sorted { fn sorted(self) -> Self; }
	impl<T: Ord> Sorted for Vec<T> {
		fn sorted(mut self) -> Self {
			self.sort();
			self
		}
	}

	#[test]
	fn map_issue_3318() {
		new_test_ext().execute_with(|| {
			OptionLinkedMap::insert(1, 1);
			assert_eq!(OptionLinkedMap::get(1), Some(1));
			OptionLinkedMap::insert(1, 2);
			assert_eq!(OptionLinkedMap::get(1), Some(2));
		});
	}

	#[test]
	fn map_swap_works() {
		new_test_ext().execute_with(|| {
			OptionLinkedMap::insert(0, 0);
			OptionLinkedMap::insert(1, 1);
			OptionLinkedMap::insert(2, 2);
			OptionLinkedMap::insert(3, 3);

			let collect = || OptionLinkedMap::iter().collect::<Vec<_>>().sorted();
			assert_eq!(collect(), vec![(0, 0), (1, 1), (2, 2), (3, 3)]);

			// Two existing
			OptionLinkedMap::swap(1, 2);
			assert_eq!(collect(), vec![(0, 0), (1, 2), (2, 1), (3, 3)]);

			// Back to normal
			OptionLinkedMap::swap(2, 1);
			assert_eq!(collect(), vec![(0, 0), (1, 1), (2, 2), (3, 3)]);

			// Left existing
			OptionLinkedMap::swap(2, 5);
			assert_eq!(collect(), vec![(0, 0), (1, 1), (3, 3), (5, 2)]);

			// Right existing
			OptionLinkedMap::swap(5, 2);
			assert_eq!(collect(), vec![(0, 0), (1, 1), (2, 2), (3, 3)]);
		});
	}

	#[test]
	fn double_map_swap_works() {
		new_test_ext().execute_with(|| {
			DataDM::insert(0, 1, 1);
			DataDM::insert(1, 0, 2);
			DataDM::insert(1, 1, 3);

			let get_all = || vec![
				DataDM::get(0, 1),
				DataDM::get(1, 0),
				DataDM::get(1, 1),
				DataDM::get(2, 0),
				DataDM::get(2, 1),
			];
			assert_eq!(get_all(), vec![1, 2, 3, 0, 0]);

			// Two existing
			DataDM::swap(0, 1, 1, 0);
			assert_eq!(get_all(), vec![2, 1, 3, 0, 0]);

			// Left existing
			DataDM::swap(1, 0, 2, 0);
			assert_eq!(get_all(), vec![2, 0, 3, 1, 0]);

			// Right existing
			DataDM::swap(2, 1, 1, 1);
			assert_eq!(get_all(), vec![2, 0, 0, 1, 3]);
		});
	}

	#[test]
	fn map_basic_insert_remove_should_work() {
		new_test_ext().execute_with(|| {
			// initialized during genesis
			assert_eq!(Map::get(&15u32), 42u64);

			// get / insert / take
			let key = 17u32;
			assert_eq!(Map::get(&key), 0u64);
			Map::insert(key, 4u64);
			assert_eq!(Map::get(&key), 4u64);
			assert_eq!(Map::take(&key), 4u64);
			assert_eq!(Map::get(&key), 0u64);

			// mutate
			Map::mutate(&key, |val| {
				*val = 15;
			});
			assert_eq!(Map::get(&key), 15u64);

			// remove
			Map::remove(&key);
			assert_eq!(Map::get(&key), 0u64);
		});
	}

	#[test]
	fn map_iteration_should_work() {
		new_test_ext().execute_with(|| {
			assert_eq!(Map::iter().collect::<Vec<_>>().sorted(), vec![(15, 42)]);
			// insert / remove
			let key = 17u32;
			Map::insert(key, 4u64);
			assert_eq!(Map::iter().collect::<Vec<_>>().sorted(), vec![(15, 42), (key, 4)]);
			assert_eq!(Map::take(&15), 42u64);
			assert_eq!(Map::take(&key), 4u64);
			assert_eq!(Map::iter().collect::<Vec<_>>().sorted(), vec![]);

			// Add couple of more elements
			Map::insert(key, 42u64);
			assert_eq!(Map::iter().collect::<Vec<_>>().sorted(), vec![(key, 42)]);
			Map::insert(key + 1, 43u64);
			assert_eq!(Map::iter().collect::<Vec<_>>().sorted(), vec![(key, 42), (key + 1, 43)]);

			// mutate
			let key = key + 2;
			Map::mutate(&key, |val| {
				*val = 15;
			});
			assert_eq!(Map::iter().collect::<Vec<_>>().sorted(), vec![(key - 2, 42), (key - 1, 43), (key, 15)]);
			Map::mutate(&key, |val| {
				*val = 17;
			});
			assert_eq!(Map::iter().collect::<Vec<_>>().sorted(), vec![(key - 2, 42), (key - 1, 43), (key, 17)]);

			// remove first
			Map::remove(&key);
			assert_eq!(Map::iter().collect::<Vec<_>>().sorted(), vec![(key - 2, 42), (key - 1, 43)]);

			// remove last from the list
			Map::remove(&(key - 2));
			assert_eq!(Map::iter().collect::<Vec<_>>().sorted(), vec![(key - 1, 43)]);

			// remove the last element
			Map::remove(&(key - 1));
			assert_eq!(Map::iter().collect::<Vec<_>>().sorted(), vec![]);
		});
	}

	#[test]
	fn double_map_basic_insert_remove_remove_prefix_should_work() {
		new_test_ext().execute_with(|| {
			type DoubleMap = DataDM;
			// initialized during genesis
			assert_eq!(DoubleMap::get(&15u32, &16u32), 42u64);

			// get / insert / take
			let key1 = 17u32;
			let key2 = 18u32;
			assert_eq!(DoubleMap::get(&key1, &key2), 0u64);
			DoubleMap::insert(&key1, &key2, &4u64);
			assert_eq!(DoubleMap::get(&key1, &key2), 4u64);
			assert_eq!(DoubleMap::take(&key1, &key2), 4u64);
			assert_eq!(DoubleMap::get(&key1, &key2), 0u64);

			// mutate
			DoubleMap::mutate(&key1, &key2, |val| {
				*val = 15;
			});
			assert_eq!(DoubleMap::get(&key1, &key2), 15u64);

			// remove
			DoubleMap::remove(&key1, &key2);
			assert_eq!(DoubleMap::get(&key1, &key2), 0u64);

			// remove prefix
			DoubleMap::insert(&key1, &key2, &4u64);
			DoubleMap::insert(&key1, &(key2 + 1), &4u64);
			DoubleMap::insert(&(key1 + 1), &key2, &4u64);
			DoubleMap::insert(&(key1 + 1), &(key2 + 1), &4u64);
			DoubleMap::remove_prefix(&key1);
			assert_eq!(DoubleMap::get(&key1, &key2), 0u64);
			assert_eq!(DoubleMap::get(&key1, &(key2 + 1)), 0u64);
			assert_eq!(DoubleMap::get(&(key1 + 1), &key2), 4u64);
			assert_eq!(DoubleMap::get(&(key1 + 1), &(key2 + 1)), 4u64);

		});
	}

	#[test]
	fn double_map_append_should_work() {
		new_test_ext().execute_with(|| {
			type DoubleMap = AppendableDM<Test>;

			let key1 = 17u32;
			let key2 = 18u32;

			DoubleMap::insert(&key1, &key2, &vec![1]);
			DoubleMap::append(&key1, &key2, 2);
			assert_eq!(DoubleMap::get(&key1, &key2), &[1, 2]);
		});
	}

	#[test]
	fn double_map_mutate_exists_should_work() {
		new_test_ext().execute_with(|| {
			type DoubleMap = DataDM;

			let (key1, key2) = (11, 13);

			// mutated
			DoubleMap::mutate_exists(key1, key2, |v| *v = Some(1));
			assert_eq!(DoubleMap::get(&key1, key2), 1);

			// removed if mutated to `None`
			DoubleMap::mutate_exists(key1, key2, |v| *v = None);
			assert!(!DoubleMap::contains_key(&key1, key2));
		});
	}

	#[test]
	fn double_map_try_mutate_exists_should_work() {
		new_test_ext().execute_with(|| {
			type DoubleMap = DataDM;
			type TestResult = result::Result<(), &'static str>;

			let (key1, key2) = (11, 13);

			// mutated if `Ok`
			assert_ok!(DoubleMap::try_mutate_exists(key1, key2, |v| -> TestResult {
				*v = Some(1);
				Ok(())
			}));
			assert_eq!(DoubleMap::get(&key1, key2), 1);

			// no-op if `Err`
			assert_noop!(DoubleMap::try_mutate_exists(key1, key2, |v| -> TestResult {
				*v = Some(2);
				Err("nah")
			}), "nah");

			// removed if mutated to`None`
			assert_ok!(DoubleMap::try_mutate_exists(key1, key2, |v| -> TestResult {
				*v = None;
				Ok(())
			}));
			assert!(!DoubleMap::contains_key(&key1, key2));
		});
	}

	const EXPECTED_METADATA: StorageMetadata = StorageMetadata {
		prefix: DecodeDifferent::Encode("Test"),
		entries: DecodeDifferent::Encode(
			&[
				StorageEntryMetadata {
					name: DecodeDifferent::Encode("Data"),
					modifier: StorageEntryModifier::Default,
					ty: StorageEntryType::Map{
						hasher: StorageHasher::Twox64Concat,
						key: DecodeDifferent::Encode("u32"),
						value: DecodeDifferent::Encode("u64"),
						unused: false,
					},
					default: DecodeDifferent::Encode(
						DefaultByteGetter(&__GetByteStructData(PhantomData::<Test>))
					),
					documentation: DecodeDifferent::Encode(&[]),
				},
				StorageEntryMetadata {
					name: DecodeDifferent::Encode("OptionLinkedMap"),
					modifier: StorageEntryModifier::Optional,
					ty: StorageEntryType::Map {
						hasher: StorageHasher::Blake2_128Concat,
						key: DecodeDifferent::Encode("u32"),
						value: DecodeDifferent::Encode("u32"),
						unused: false,
					},
					default: DecodeDifferent::Encode(
						DefaultByteGetter(&__GetByteStructOptionLinkedMap(PhantomData::<Test>))
					),
					documentation: DecodeDifferent::Encode(&[]),
				},
				StorageEntryMetadata {
					name: DecodeDifferent::Encode("GenericData"),
					modifier: StorageEntryModifier::Default,
					ty: StorageEntryType::Map{
						hasher: StorageHasher::Identity,
						key: DecodeDifferent::Encode("T::BlockNumber"),
						value: DecodeDifferent::Encode("T::BlockNumber"),
						unused: false
					},
					default: DecodeDifferent::Encode(
						DefaultByteGetter(&__GetByteStructGenericData(PhantomData::<Test>))
					),
					documentation: DecodeDifferent::Encode(&[]),
				},
				StorageEntryMetadata {
					name: DecodeDifferent::Encode("GenericData2"),
					modifier: StorageEntryModifier::Optional,
					ty: StorageEntryType::Map{
						hasher: StorageHasher::Blake2_128Concat,
						key: DecodeDifferent::Encode("T::BlockNumber"),
						value: DecodeDifferent::Encode("T::BlockNumber"),
						unused: false
					},
					default: DecodeDifferent::Encode(
						DefaultByteGetter(&__GetByteStructGenericData2(PhantomData::<Test>))
					),
					documentation: DecodeDifferent::Encode(&[]),
				},
				StorageEntryMetadata {
					name: DecodeDifferent::Encode("DataDM"),
					modifier: StorageEntryModifier::Default,
					ty: StorageEntryType::DoubleMap{
						hasher: StorageHasher::Twox64Concat,
						key1: DecodeDifferent::Encode("u32"),
						key2: DecodeDifferent::Encode("u32"),
						value: DecodeDifferent::Encode("u64"),
						key2_hasher: StorageHasher::Blake2_128Concat,
					},
					default: DecodeDifferent::Encode(
						DefaultByteGetter(&__GetByteStructDataDM(PhantomData::<Test>))
					),
					documentation: DecodeDifferent::Encode(&[]),
				},
				StorageEntryMetadata {
					name: DecodeDifferent::Encode("GenericDataDM"),
					modifier: StorageEntryModifier::Default,
					ty: StorageEntryType::DoubleMap{
						hasher: StorageHasher::Blake2_128Concat,
						key1: DecodeDifferent::Encode("T::BlockNumber"),
						key2: DecodeDifferent::Encode("T::BlockNumber"),
						value: DecodeDifferent::Encode("T::BlockNumber"),
						key2_hasher: StorageHasher::Identity,
					},
					default: DecodeDifferent::Encode(
						DefaultByteGetter(&__GetByteStructGenericDataDM(PhantomData::<Test>))
					),
					documentation: DecodeDifferent::Encode(&[]),
				},
				StorageEntryMetadata {
					name: DecodeDifferent::Encode("GenericData2DM"),
					modifier: StorageEntryModifier::Optional,
					ty: StorageEntryType::DoubleMap{
						hasher: StorageHasher::Blake2_128Concat,
						key1: DecodeDifferent::Encode("T::BlockNumber"),
						key2: DecodeDifferent::Encode("T::BlockNumber"),
						value: DecodeDifferent::Encode("T::BlockNumber"),
						key2_hasher: StorageHasher::Twox64Concat,
					},
					default: DecodeDifferent::Encode(
						DefaultByteGetter(&__GetByteStructGenericData2DM(PhantomData::<Test>))
					),
					documentation: DecodeDifferent::Encode(&[]),
				},
				StorageEntryMetadata {
					name: DecodeDifferent::Encode("AppendableDM"),
					modifier: StorageEntryModifier::Default,
					ty: StorageEntryType::DoubleMap{
						hasher: StorageHasher::Blake2_128Concat,
						key1: DecodeDifferent::Encode("u32"),
						key2: DecodeDifferent::Encode("T::BlockNumber"),
						value: DecodeDifferent::Encode("Vec<u32>"),
						key2_hasher: StorageHasher::Blake2_128Concat,
					},
					default: DecodeDifferent::Encode(
						DefaultByteGetter(&__GetByteStructGenericData2DM(PhantomData::<Test>))
					),
					documentation: DecodeDifferent::Encode(&[]),
				},
			]
		),
	};

	#[test]
	fn store_metadata() {
		let metadata = Module::<Test>::storage_metadata();
		pretty_assertions::assert_eq!(EXPECTED_METADATA, metadata);
	}

	parameter_types! {
		storage StorageParameter: u64 = 10;
	}

	#[test]
	fn check_storage_parameter_type_works() {
		TestExternalities::default().execute_with(|| {
			assert_eq!(sp_io::hashing::twox_128(b":StorageParameter:"), StorageParameter::key());

			assert_eq!(10, StorageParameter::get());

			StorageParameter::set(&300);
			assert_eq!(300, StorageParameter::get());
		})
	}
}

/// prelude to be used alongside pallet macro, for ease of use.
pub mod pallet_prelude {
	pub use sp_std::marker::PhantomData;
	pub use frame_support::traits::{Get, Interface, IsType};
	#[cfg(feature = "std")]
	pub use frame_support::traits::GenesisBuild;
	pub use frame_support::dispatch::{DispatchResultWithPostInfo, Parameter};
	pub use frame_support::storage::types::*;
	pub use frame_support::{EqNoBound, PartialEqNoBound, DebugStripped, DebugNoBound, CloneNoBound};
	pub use codec::{Encode, Decode};
	pub use sp_inherents::ProvideInherent;
	pub use sp_inherents::InherentData;
	pub use sp_inherents::InherentIdentifier;
	pub use crate::weights::Weight;
	pub use crate::{
		Twox256, Twox128, Blake2_256, Blake2_128, Identity, Twox64Concat, Blake2_128Concat,
	};
	pub use sp_runtime::{
		traits::{MaybeSerializeDeserialize, Member, ValidateUnsigned},
		transaction_validity::{
			TransactionSource, TransactionValidity, ValidTransaction, TransactionPriority,
			TransactionTag, TransactionLongevity, TransactionValidityError, InvalidTransaction,
			UnknownTransaction,
		},
	};
	pub use crate::{
		StorageValue, StorageMap, StorageDoubleMap, StoragePrefixedMap, IterableStorageMap,
		IterableStorageDoubleMap,
	};
}

/// `pallet` attribute macro allows to define a pallet to be used in `construct_runtime!`.
///
/// It is define by a module item:
/// ```nocompile
/// #[pallet]
/// mod pallet {
/// ...
/// }
/// ```
///
/// Inside the module the macro will parse item with the attribute: `#[pallet::*]`, some attributes
/// are mandatory, some other optional.
///
/// The attribute are explained with the syntax of non instantiable pallets, to see how pallet with
/// instance work see below example.
///
/// Note various type can be automatically imported using pallet_prelude in frame_support and
/// frame_system:
/// ```nocompile
/// #[pallet]
/// mod {
///		use frame_support::pallet_prelude::*;
///		use frame_system::pallet_prelude::*;
///		...
/// }
/// ```
///
/// # Config trait: `#[pallet::config]` mandatory
///
/// The trait defining generics of the pallet.
///
/// Item must be defined as
/// ```nocompile
/// #[pallet::config]
/// pub trait Config: frame_system::Config + $optionally_some_other_supertraits {
/// ...
/// }
/// ```
/// I.e. a regular trait definition named `Config`, with supertrait `frame_system::Config`,
/// optionally other supertrait but no where clause.
///
/// The associated type `Event` is reserved, if defined it must bounds `From<Event>` and
/// `IsType<<Self as frame_system::Config>::Event>`, see `#[pallet::event]` for more information.
///
/// To put `Get` associated type into metadatas, use the attribute `#[pallet::constant]`, e.g.:
/// ```nocompile
/// #[pallet::config]
/// pub trait Config: frame_system::Config {
///		#[pallet::constant]
///		type Foo: Get<u32>;
/// }
/// ```
///
/// To bypass the `frame_system::Config` supertrait check, use the attribute
/// `#[pallet::disable_frame_system_supertrait_check]`, e.g.:
/// ```nocompile
/// #[pallet::config]
/// #[pallet::disable_frame_system_supertrait_check]
/// pub trait Config: pallet_timestamp::Config {}
/// ```
///
/// NOTE: At this moment macro doesn't support where clause on trait, but where clause can be
/// used on some of the usage of this trait.
///
/// ### Macro expansion:
///
/// The macro expand pallet constant metadata with the information given by `#[pallet::constant]`.
///
/// # Pallet struct placeholder: `#[pallet::pallet]` mandatory
///
/// The placeholder struct, on which is implemented pallet informations.
///
/// Item must be defined as followed:
/// ```nocompile
/// #[pallet::pallet]
/// pub struct Pallet<T>(PhantomData<T>);
/// ```
/// I.e. a regular struct definition named `Pallet`, with generic T and no where clause.
///
/// To generate a `Store` trait associating all storages, use the attribute
/// `#[pallet::generate_store($vis trait Store)]`, e.g.:
/// ```nocompile
/// #[pallet::pallet]
/// #[pallet::generate_store(pub(super) trait Store)]
/// pub struct Pallet<T>(PhantomData<T>);
/// ```
/// More precisely the store trait contains an associated type for each storage. It is implemented
/// for `Pallet` allowing to access the storage from pallet struct.
///
/// ### Macro expansion:
///
/// The macro add this attribute to the struct definition:
/// ```nocompile
/// #[derive(
/// 	frame_support::CloneNoBound,
/// 	frame_support::EqNoBound,
/// 	frame_support::PartialEqNoBound,
/// 	frame_support::DebugStripped,
/// )]
/// ```
///
/// If attribute generate_store then macro create the trait `Store` and implement it on `Pallet`.
///
/// # Interface: `#[pallet::interface]` mandatory
///
/// Implementation of `Interface` on `Pallet` allowing to define some specific pallet logic.
///
/// Item must be defined as
/// ```nocompile
/// #[pallet::interface]
/// impl<T: Config> Interface<BlockNumberFor<T>> for Pallet<T> $optional_where_clause {
/// }
/// ```
/// I.e. a regular trait implementation with generic bound: `T: Config`, for the trait
/// `Interface<BlockNumberFor<T>>` (they are defined in preludes), for the type `Pallet<T>`
/// and with an optional where clause.
///
/// ### Macro expansion:
///
/// The macro implement the traits `OnInitialize`, `OnFinalize`, `OnRuntimeUpgrade`,
/// `OffchainWorker`, `IntegrityTest` using `Interface` implementation.
///
/// # Call: `#[pallet::call]` mandatory
///
/// Implementation of pallet dispatchables.
///
/// Item must be defined as:
/// ```nocompile
/// #[pallet::call]
/// impl<T: Config> Pallet<T> {
/// 	/// $some_doc
/// 	#[pallet::weight($ExpressionResultingInWeight)]
/// 	$vis fn $fn_name(
/// 		origin: OriginFor<T>,
/// 		$some_arg: $some_type,
/// 		// or with compact attribute: #[pallet::compact] $some_arg: $some_type,
/// 		...
/// 	) -> DispatchResultWithPostInfo {
/// 		...
/// 	}
/// 	...
/// }
/// ```
/// I.e. a regular type implementation, with generic `T: Config`, on type `Pallet<T>`, with
/// optional where clause.
///
/// Each dispatchable needs to define a weight with `#[pallet::weight($expr)]` attribute,
/// the first argument must be `origin: OriginFor<T>`, compact encoding for argument can be used
/// using `#[pallet::compact]`, function must return DispatchResultWithPostInfo.
///
/// All arguments must implement `Debug`, `PartialEq`, `Eq`, `Decode`, `Encode`, `Clone`. For ease
/// of use just bound trait `Member` available in frame_support::pallet_prelude.
///
/// **WARNING**: modifying dispatchables, changing their order, removing some must be done with
/// care. Indeed this will change the outer runtime call type (which is an enum with one variant
/// per pallet), this outer runtime call can be stored on-chain (e.g. in pallet-scheduler).
/// Thus migration might be needed.
///
/// ### Macro expansion
///
/// The macro create an enum `Call` with one variant per dispatchable. This enum implements:
/// `Clone`, `Eq`, `PartialEq`, `Debug` (with stripped implementation in `not("std")`), `Encode`,
/// `Decode`, `GetDispatchInfo`, `GetCallName`, `UnfilteredDispatchable`.
///
/// The macro implement on `Pallet`, the `Callable` trait and a function `call_functions` which
/// returns the dispatchable metadatas.
///
/// # Error: `#[pallet::error]` optional
///
/// Allow to define an error type to be return from dispatchable on error.
/// This error type informations are put into metadata.
///
/// Item must be defined as:
/// ```nocompile
/// #[pallet::error]
/// pub enum Error<T> {
/// 	/// $some_optional_doc
/// 	$SomeFieldLessVariant,
/// 	...
/// }
/// ```
/// I.e. a regular rust enum named `Error`, with generic `T` and fieldless variants.
///
/// ### Macro expansion
///
/// The macro implements `Debug` trait and functions `as_u8` using variant position, and `as_str`
/// using variant doc.
///
/// The macro implements `From<Error<T>>` for `&'static str`.
/// The macro implements `From<Error<T>>` for `DispatchError`.
///
/// The macro implements `ModuleErrorMetadata` on `Pallet` defining the `ErrorMetadata` of the
/// pallet.
///
/// # Event: `#[pallet::event]` optional
///
/// Allow to define pallet events, pallet events are stored in the block when they deposited (and
/// removed in next block).
///
/// Item is defined as:
/// ```nocompile
/// #[pallet::event]
/// #[pallet::metadata($SomeType = $Metadata, $SomeOtherType = $Metadata, ..)] // Optional
/// #[pallet::generate($visbility fn deposit_event)] // Optional
/// pub enum Event<$some_generic> {
/// 	/// Some doc
/// 	$SomeName($SomeType, $YetanotherType, ...),
/// 	...
/// }
/// ```
/// I.e. an enum (with named or unnamed fields variant), named Event, with generic: none or `T` or
/// `T: Config`, and without where clause.
///
/// Each field must implement `Clone`, `Eq`, `PartialEq`, `Encode`, `Decode`, and `Debug` (on std
/// only).
/// For ease of use just bound trait `Member` available in frame_support::pallet_prelude.
///
/// Variant documentations and field types are put into metadata.
/// The attribute `#[pallet::metadata(..)]` allows to specify the metadata to put for some types.
///
/// The metadat of a type is defined by:
/// * if matching a type in `#[pallet::metadata(..)]`, then the corresponding metadata.
/// * otherwise the last segment of the type.
///
/// E.g.:
/// ```nocompile
/// #[pallet::event]
/// #[pallet::metadata(u32 = SpecialU32)]
/// pub enum Event<T: Config> {
/// 	Proposed(u32, T::AccountId),
/// }
/// ```
/// will write in event variant metadata `"SpecialU32"` and `"AccountId"`.
///
/// The attribute `#[pallet::generate($visbility fn deposit_event)]` generate a helper function on
/// `Pallet` to deposit event.
///
/// NOTE: For instantiable pallet, event must be generic over T and I.
///
/// NOTE: At this moment declaration doesn't support where clause.
///
/// ### Macro expansion:
///
/// Macro will add on enum `Event` the attributes:
/// * `#[derive(frame_support::CloneNoBound)]`,
/// * `#[derive(frame_support::EqNoBound)]`,
/// * `#[derive(frame_support::PartialEqNoBound)]`,
/// * `#[derive(codec::Encode)]`,
/// * `#[derive(codec::Decode)]`,
/// * `#[cfg_attr(not(feature = "std"), derive(frame_support::DebugStripped))]`
/// * `#[cfg_attr(feature = "std", derive(frame_support::DebugNoBound))]`
///
/// Macro implements `From<Event<..>>` for ().
///
/// Macro implements metadata function on `Event` returning the `EventMetadata`.
///
/// If `#[pallet::generate]` then macro implement `fn deposit_event` on `Pallet`.
///
/// # Storage: `#[pallet::storage]` optional
///
/// Allow to define some abstract storage inside runtime storage and also set its metadata.
/// This attribute can be used multiple times.
///
/// Item is defined as:
/// ```nocompile
/// #[pallet::storage]
/// #[pallet::generate_getter(fn $getter_name)] // optional
/// $vis type $StorageName<$some_generic> = $StorageType<_, $some_generics, ...>;
/// ```
/// I.e. it must be a type alias, with generics: `T` or `T: Config`, aliased type must be one
/// of `StorageValueType`, `StorageMapType` or `StorageDoubleMapType` (defined in frame_support).
/// Their first generic must be `_` as it is written by the macro itself.
///
/// The Prefix generic written by the macro is generated using PalletInfo::name and the name of the
/// storage type.
/// E.g. if runtime name the pallet MyExample the storage `type Foo<T> = ...` use
/// prefixes: `Twox128(b"MyExample") ++ Twox128(b"Foo")`.
///
/// The optional attribute `#[pallet::generate_getter(fn $my_getter_fn_name)]` allow to define a
/// getter function on `Pallet`.
///
/// E.g:
/// ```nocompile
/// #[pallet::storage]
/// #[pallet::generate_getter(fn my_storage)]
/// pub(super) type MyStorage<T> = StorageMapType<_, Blake2_128Concat, u32, u32>;
/// ```
///
/// NOTE: if the querykind generic parameter is still generic at this stage or is using some type
/// alias then the generation of the getter might fail. In this case getter can be implemented
/// manually.
///
/// ### Macro expansion
///
/// For each storage the macro generate a struct named
/// `_GeneratedPrefixForStorage$NameOfStorage`, implements StorageInstance on it using pallet
/// names and storage name. and use it as first generic of the aliased type.
///
/// The macro implement the function `storage_metadata` on `Pallet` implementing the metadata for
/// storages.
///
/// # Type value: `#[pallet::type_value]` optional
///
/// Helper to define a struct implementing `Get` trait. To ease use of storage types.
/// This attribute can be used multiple time.
///
/// Item is defined as
/// ```nocompile
/// #[pallet::type_value]
/// fn $MyDefaultName<$some_generic>() -> $default_type { $expr }
/// ```
/// I.e.: a function definition with generics none or `T: Config` and a returned type.
///
/// E.g.:
/// ```nocompile
/// #[pallet::type_value]
/// fn MyDefault<T: Config>() -> T::Balance { 3.into() }
/// ```
///
/// NOTE: This attribute is meant to be used alongside `#[pallet::storage]` to defined some
/// specific default value in storage.
///
/// ### Macro expansion
///
/// Macro generate struct with the name of the function and its generic, and implement
/// `Get<$ReturnType>` on it using the provided function block.
///
/// # Genesis config: `#[pallet::genesis_config]` optional
///
/// Allow to define the genesis configuration of the pallet.
///
/// Item is defined as either a type alias or an enum or a struct.
/// It needs to be public and implement trait GenesisBuild with `#[pallet::genesis_build]`.
/// The type generics is constrained to be either none, or `T` or `T: Config`.
///
/// E.g:
/// ```nocompile
/// #[pallet::genesis_config]
/// pub struct GenesisConfig<T: Config> {
/// 	_myfield: BalanceOf<T>,
/// }
/// ```
///
/// ### Macro expansion
///
/// Macro will add the following attribute on it:
/// * `#[cfg(feature = "std")]`
/// * `#[derive(Serialize, Deserialize)]`
/// * `#[serde(rename_all = "camelCase")]`
/// * `#[serde(deny_unknown_fields)]`
/// * `#[serde(bound(serialize = ""))]`
/// * `#[serde(bound(deserialize = ""))]`
///
/// # Genesis build: `#[pallet::genesis_build]` optional
///
/// Allow to define how genesis_configuration is built.
///
/// Item is defined as
/// ```nocompile
/// #[pallet::genesis_build]
/// impl<T: Config> GenesisBuild<T> for GenesisConfig<$maybe_generics> {
/// 	fn build(&self) { $expr }
/// }
/// ```
/// I.e. a rust trait implementation with generic `T: Config`, of trait `GenesisBuild<T>` on type
/// `GenesisConfig` with generics none or `T`.
///
/// E.g.:
/// ```nocompile
/// #[pallet::genesis_build]
/// impl<T: Config> GenesisBuild<T> for GenesisConfig {
/// 	fn build(&self) {}
/// }
/// ```
///
/// ### Macro expansion
///
/// Macro will add the following attribute on it:
/// * `#[cfg(feature = "std")]`
///
/// Macro will implement `sp_runtime::BuildModuleGenesisStorage` using `()` as second generic for
/// non-instantiable pallets.
///
/// # Inherent: `#[pallet::inherent]` optional
///
/// Allow the pallet to provide some inherent:
///
/// Item is defined as:
/// ```nocompile
/// #[pallet::inherent]
/// impl<T: Config> ProvideInherent for Pallet<T> {
/// 	// ... regular trait implementation
/// }
/// ```
/// I.e. a trait implementation with bound `T: Config`, of trait `ProvideInherent` for type
/// `Pallet<T>`, and some optional where clause.
///
/// ### Macro expansion
///
/// Macro make currenlty no use of this information, but it might use this information in the
/// future to give information directly to construct_runtime.
///
/// # Validate unsigned: `#[pallet::validate_unsigned]` optional
///
/// Allow the pallet to validate some unsigned transaction:
///
/// Item is defined as:
/// ```nocompile
/// #[pallet::validate_unsigned]
/// impl<T: Config> ValidateUnsigned for Pallet<T> {
/// 	// ... regular trait implementation
/// }
/// ```
/// I.e. a trait implementation with bound `T: Config`, of trait `ValidateUnsigned` for type
/// `Pallet<T>`, and some optional where clause.
///
/// ### Macro expansion
///
/// Macro make currenlty no use of this information, but it might use this information in the
/// future to give information directly to construct_runtime.
///
/// # Origin: `#[pallet::origin]` optional
///
/// Allow to define some origin for the pallet.
///
/// Item must be either a type alias or an enum or a struct. It needs to be public.
///
/// E.g.:
/// ```nocompile
/// #[pallet::origin]
/// pub struct Origin<T>(PhantomData<(T)>);
/// ```
///
/// **WARNING**: modifying origin changes the outer runtime origin. This outer runtime origin can
/// be stored on-chain (e.g. in pallet-scheduler), thus any change must be done with care as it
/// might require some migration.
///
/// NOTE: for instantiable pallet, origin must be generic over T and I.
///
/// # Example for pallet without instance.
///
/// ```
/// #[frame_support::pallet]
/// // NOTE: Example is name of the pallet, it will be used as unique identifier for storage
/// pub mod pallet {
/// 	use frame_support::pallet_prelude::*; // Import various types used in pallet definition
/// 	use frame_system::pallet_prelude::*; // OriginFor helper type for implementing dispatchables.
///
/// 	type BalanceOf<T> = <T as Config>::Balance;
///
/// 	// Define the generic parameter of the pallet
/// 	// The macro checks trait generics: is expected none or `I = ()`.
/// 	// The macro parses `#[pallet::constant]` attributes: used to generate constant metadata,
/// 	// expected syntax is `type $IDENT: Get<$TYPE>;`.
/// 	#[pallet::config]
/// 	pub trait Config: frame_system::Config {
/// 		#[pallet::constant] // put the constant in metadata
/// 		type MyGetParam: Get<u32>;
/// 		type Balance: Parameter + From<u8>;
/// 		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
/// 	}
///
/// 	// Define the pallet struct placeholder, various pallet function are implemented on it.
/// 	// The macro checks struct generics: is expected `T` or `T, I = DefaultInstance`
/// 	#[pallet::pallet]
/// 	#[pallet::generate_store(pub(super) trait Store)]
/// 	pub struct Pallet<T>(PhantomData<T>);
///
/// 	// Implement on the pallet interface on pallet.
/// 	// The macro checks:
/// 	// * trait is `Interface` (imported from pallet_prelude)
/// 	// * struct is `Pallet<T>` or `Pallet<T, I>`
/// 	#[pallet::interface]
/// 	impl<T: Config> Interface<BlockNumberFor<T>> for Pallet<T> {
/// 	}
///
/// 	// Declare Call struct and implement dispatchables.
/// 	//
/// 	// WARNING: Each parameter used in functions must implement: Clone, Debug, Eq, PartialEq,
/// 	// Codec.
/// 	//
/// 	// The macro checks:
/// 	// * pallet is `Pallet<T>` or `Pallet<T, I>`
/// 	// * trait is `Call`
/// 	// * each dispatchable functions first argument is `origin: OriginFor<T>` (OriginFor is
/// 	//   imported from frame_system.
/// 	//
/// 	// The macro parse `#[pallet::compact]` attributes, function parameter with this attribute
/// 	// will be encoded/decoded using compact codec in implementation of codec for the enum
/// 	// `Call`.
/// 	//
/// 	// The macro generate the enum `Call` with a variant for each dispatchable and implements
/// 	// codec, Eq, PartialEq, Clone and Debug.
/// 	#[pallet::call]
/// 	impl<T: Config> Pallet<T> {
/// 		/// Doc comment put in metadata
/// 		#[pallet::weight(0)] // Defines weight for call (function parameters are in scope)
/// 		fn toto(
/// 			origin: OriginFor<T>,
/// 			#[pallet::compact] _foo: u32
/// 		) -> DispatchResultWithPostInfo {
/// 			let _ = origin;
/// 			unimplemented!();
/// 		}
/// 	}
///
/// 	// Declare pallet Error enum. (this is optional)
/// 	// The macro checks enum generics and that each variant is unit.
/// 	// The macro generate error metadata using doc comment on each variant.
/// 	#[pallet::error]
/// 	pub enum Error<T> {
/// 		/// doc comment put into metadata
/// 		InsufficientProposersBalance,
/// 	}
///
/// 	// Declare pallet Event enum. (this is optional)
/// 	//
/// 	// WARNING: Each type used in variants must implement: Clone, Debug, Eq, PartialEq, Codec.
/// 	//
/// 	// The macro generates event metadata, and derive Clone, Debug, Eq, PartialEq and Codec
/// 	#[pallet::event]
/// 	// Additional argument to specify the metadata to use for given type.
/// 	#[pallet::metadata(BalanceOf<T> = Balance, u32 = Other)]
/// 	// Generate a funciton on Pallet to deposit an event.
/// 	#[pallet::generate(pub(super) fn deposit_event)]
/// 	pub enum Event<T: Config> {
/// 		/// doc comment put in metadata
/// 		// `<T as frame_system::Config>::AccountId` is not defined in metadata list, the last
/// 		// segment is put into metadata, i.e. `AccountId`
/// 		Proposed(<T as frame_system::Config>::AccountId),
/// 		/// doc
/// 		// here metadata will be `Balance` as define in metadata list
/// 		Spending(BalanceOf<T>),
/// 		// here metadata will be `Other` as define in metadata list
/// 		Something(u32),
/// 	}
///
/// 	// Define a struct which implements `frame_support::traits::Get<T::Balance>`
/// 	#[pallet::type_value]
/// 	pub(super) fn MyDefault<T: Config>() -> T::Balance { 3.into() }
///
/// 	// Declare a storage, any amount of storage can be declared.
/// 	//
/// 	// Is expected either `StorageValueType`, `StorageMapType` or `StorageDoubleMapType`.
/// 	// The macro generates for struct `$identP` (for storage of name `$ident`) and implement
/// 	// storage instance on it.
/// 	// The macro macro expand the metadata for the storage with the type used:
/// 	// * For storage value the type for value will be copied into metadata
/// 	// * For storage map the type for value and the type for key will be copied into metadata
/// 	// * For storage double map the type for value, key1, and key2 will be copied into
/// 	//   metadata.
/// 	//
/// 	// NOTE: for storage hasher, the type is not copied because storage hasher trait already
/// 	// implements metadata. Thus generic storage hasher is supported.
/// 	#[pallet::storage]
/// 	pub(super) type MyStorageValue<T: Config> =
/// 		StorageValueType<_, T::Balance, ValueQuery, MyDefault<T>>;
///
/// 	// Another declaration
/// 	#[pallet::storage]
/// 	#[pallet::generate_getter(fn my_storage)]
/// 	pub(super) type MyStorage<T> = StorageMapType<_, Blake2_128Concat, u32, u32>;
///
/// 	// Declare genesis config. (This is optional)
/// 	//
/// 	// The macro accept either type alias or struct or enum, it checks generics are consistent.
/// 	//
/// 	// Type must implement `Default` traits
/// 	#[pallet::genesis_config]
/// 	#[derive(Default)]
/// 	pub struct GenesisConfig {
/// 		_myfield: u32,
/// 	}
///
/// 	// Declare genesis builder. (This is need only if GenesisConfig is declared)
/// 	#[pallet::genesis_build]
/// 	impl<T: Config> GenesisBuild<T> for GenesisConfig {
/// 		fn build(&self) {}
/// 	}
///
/// 	// Declare a pallet origin. (this is optional)
/// 	//
/// 	// The macro accept type alias or struct or enum, it checks generics are consistent.
/// 	#[pallet::origin]
/// 	pub struct Origin<T>(PhantomData<T>);
///
/// 	// Declare validate_unsigned implementation.
/// 	#[pallet::validate_unsigned]
/// 	impl<T: Config> ValidateUnsigned for Pallet<T> {
/// 		type Call = Call<T>;
/// 		fn validate_unsigned(
/// 			source: TransactionSource,
/// 			call: &Self::Call
/// 		) -> TransactionValidity {
/// 			Err(TransactionValidityError::Invalid(InvalidTransaction::Call))
/// 		}
/// 	}
///
/// 	// Declare inherent provider for pallet. (this is optional)
/// 	//
/// 	// The macro checks pallet is `Pallet<T>` or `Pallet<T, I>` and trait is `ProvideInherent`
/// 	#[pallet::inherent]
/// 	impl<T: Config> ProvideInherent for Pallet<T> {
/// 		type Call = Call<T>;
/// 		type Error = InherentError;
///
/// 		const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;
///
/// 		fn create_inherent(_data: &InherentData) -> Option<Self::Call> {
/// 			unimplemented!();
/// 		}
/// 	}
///
/// 	// Regular rust code needed for implementing ProvideInherent trait
///
/// 	#[derive(codec::Encode, sp_runtime::RuntimeDebug)]
/// 	#[cfg_attr(feature = "std", derive(codec::Decode))]
/// 	pub enum InherentError {
/// 	}
///
/// 	impl sp_inherents::IsFatalError for InherentError {
/// 		fn is_fatal_error(&self) -> bool {
/// 			unimplemented!();
/// 		}
/// 	}
///
/// 	pub const INHERENT_IDENTIFIER: sp_inherents::InherentIdentifier = *b"testpall";
/// }
/// ```
///
/// # Example for pallet with instance.
///
/// ```
/// #[frame_support::pallet]
/// pub mod pallet {
/// 	use frame_support::pallet_prelude::*;
/// 	use frame_system::pallet_prelude::*;
///
/// 	type BalanceOf<T, I = ()> = <T as Config<I>>::Balance;
///
/// 	#[pallet::config]
/// 	pub trait Config<I: 'static = ()>: frame_system::Config {
/// 		#[pallet::constant]
/// 		type MyGetParam: Get<u32>;
/// 		type Balance: Parameter + From<u8>;
/// 		type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;
/// 	}
///
/// 	#[pallet::pallet]
/// 	#[pallet::generate_store(pub(super) trait Store)]
/// 	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);
///
/// 	#[pallet::interface]
/// 	impl<T: Config<I>, I: 'static> Interface<BlockNumberFor<T>> for Pallet<T, I> {
/// 	}
///
/// 	#[pallet::call]
/// 	impl<T: Config<I>, I: 'static> Pallet<T, I> {
/// 		/// Doc comment put in metadata
/// 		#[pallet::weight(0)]
/// 		fn toto(origin: OriginFor<T>, #[pallet::compact] _foo: u32) -> DispatchResultWithPostInfo {
/// 			let _ = origin;
/// 			unimplemented!();
/// 		}
/// 	}
///
/// 	#[pallet::error]
/// 	pub enum Error<T, I = ()> {
/// 		/// doc comment put into metadata
/// 		InsufficientProposersBalance,
/// 	}
///
/// 	#[pallet::event]
/// 	#[pallet::metadata(BalanceOf<T> = Balance, u32 = Other)]
/// 	#[pallet::generate(pub(super) fn deposit_event)]
/// 	pub enum Event<T: Config<I>, I: 'static = ()> {
/// 		/// doc comment put in metadata
/// 		Proposed(<T as frame_system::Config>::AccountId),
/// 		/// doc
/// 		Spending(BalanceOf<T, I>),
/// 		Something(u32),
/// 	}
///
/// 	#[pallet::type_value]
/// 	pub(super) fn MyDefault<T: Config<I>, I: 'static>() -> T::Balance { 3.into() }
///
/// 	#[pallet::storage]
/// 	pub(super) type MyStorageValue<T: Config<I>, I: 'static = ()> =
/// 		StorageValueType<_, T::Balance, ValueQuery, MyDefault<T, I>>;
///
/// 	#[pallet::storage]
/// 	#[pallet::generate_getter(fn my_storage)]
/// 	pub(super) type MyStorage<T, I = ()> =
/// 		StorageMapType<_, Blake2_128Concat, u32, u32>;
///
/// 	#[pallet::genesis_config]
/// 	#[derive(Default)]
/// 	pub struct GenesisConfig {
/// 		_myfield: u32,
/// 	}
///
/// 	#[pallet::genesis_build]
/// 	impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig {
/// 		fn build(&self) {}
/// 	}
///
/// 	#[pallet::origin]
/// 	pub struct Origin<T, I = ()>(PhantomData<(T, I)>);
///
/// 	#[pallet::validate_unsigned]
/// 	impl<T: Config<I>, I: 'static> ValidateUnsigned for Pallet<T, I> {
/// 		type Call = Call<T, I>;
/// 		fn validate_unsigned(
/// 			source: TransactionSource,
/// 			call: &Self::Call
/// 		) -> TransactionValidity {
/// 			Err(TransactionValidityError::Invalid(InvalidTransaction::Call))
/// 		}
/// 	}
///
/// 	#[pallet::inherent]
/// 	impl<T: Config<I>, I: 'static> ProvideInherent for Pallet<T, I> {
/// 		type Call = Call<T, I>;
/// 		type Error = InherentError;
///
/// 		const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;
///
/// 		fn create_inherent(_data: &InherentData) -> Option<Self::Call> {
/// 			unimplemented!();
/// 		}
/// 	}
///
/// 	// Regular rust code needed for implementing ProvideInherent trait
///
/// 	#[derive(codec::Encode, sp_runtime::RuntimeDebug)]
/// 	#[cfg_attr(feature = "std", derive(codec::Decode))]
/// 	pub enum InherentError {
/// 	}
///
/// 	impl sp_inherents::IsFatalError for InherentError {
/// 		fn is_fatal_error(&self) -> bool {
/// 			unimplemented!();
/// 		}
/// 	}
///
/// 	pub const INHERENT_IDENTIFIER: sp_inherents::InherentIdentifier = *b"testpall";
/// }
/// ```
///
/// # Notes for migration from decl_* macros
///
/// A good strategy is to start from the code below and add decl_module, decl_storage, decl_event,
/// ValidateUsigned, Inherent information inside the pallet.
///
/// ```nocompile
/// pub use pallet::*;
/// #[frame_support::pallet]
/// mod pallet {
///		use frame_support::pallet_prelude::*;
///		use frame_system::pallet_prelude::*;
///		use super::*;
///
///		#[pallet::pallet]
///		#[pallet::generate_store(pub(super) Store)]
///		pub struct Pallet<T>(PhantomData<T>);
/// }
/// ```
///
/// * macros defined pallet types in scope thus in order not to break the crate you might want to
///   reexport pallets types, and import crate types.
///
///   ```nocompile
///   pub use pallet::*;
///   mod pallet {
///   	use crate::*;
///   	...
///   }
///   ```
///
/// * storages are now defined in a module, to make available private storages in crate scope
///   use `pub(super) type MyStorage = ..`
///
/// * dispatch functions are now defined in a module, to make available private storages in \
///   crate scope use `pub(super) fn foo(origin...`
///
/// * storages attributes: `get(fn my_getter)` should now be written:
///   `#[pallet::generate_getter(fn my_getter)]`
///
/// * storages attributes: build and config have been removed, they must be manually written in the
///   GenesisConfig and GenesisBuild implementation.
///
/// * storages which value is `Option<$SomeType>` is now of value `$SomeType` and QueryKind is
///   `OptionQuery`. storages which value is `$SomeType` (not an option) is of QueryKind
///   `ValueQuery`.
///   Thus `MyStorage: u32` translate to `type MyStorage = StorageValueType<_, u32, ValueQuery>`
///   Thus `MyStorage: Option<u32>` translate to
///   `type MyStorage = StorageValueType<_, u32>` (OptionQuery being default).
///
/// * dispatchable function must now have the full signature, thus origin can be replace by
///   `origin: OriginFor<T>` and result must be `DispatchResultWithPostInfo`, thus `Ok(().into()`
///   can be added as result.
///
/// * `#[compact]` on dispatchable argument must be replaced by `#[pallet::compact]`
///
/// * `#[weight = ..]` replaced by `#[pallet::weight(..)]`
///
/// * `RawEvent` is no longer generated thus replaced by `Event`.
///
/// * `on_initialize`, `on_finalize`, `on_runtime_upgrade`, `offchain_worker`, `integrity_test` are
///   moved to `Interface implementation.
pub use frame_support_procedural::pallet;
