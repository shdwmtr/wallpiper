// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    borrow::Cow,
    cell::RefCell,
    collections::BTreeMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use once_cell::unsync::OnceCell;

/// The [CheckEmpty] trait allows to check if a collection is empty.
pub trait CheckEmpty: Default + Clone {
    /// Returns `true` if the collection contains no elements.
    fn is_empty(&self) -> bool;
}

impl<T: Clone> CheckEmpty for Vec<T> {
    fn is_empty(&self) -> bool {
        Vec::<T>::is_empty(self)
    }
}

impl<K: Clone, V: Clone> CheckEmpty for BTreeMap<K, V> {
    fn is_empty(&self) -> bool {
        BTreeMap::<K, V>::is_empty(self)
    }
}

/// A collection wrapper, that guarantees that an empty collection can be trivially destructed.
///
/// This makes it easy to use collections in a global object that requires trivially destructible.
/// When using global objects in a dynamic link library that allow a process to load and unload the
/// dynamic link library multiple times, no OS provides reliable way to teardown the global
/// resources allocated, so all global resources used should be trivially destructible.
///
/// This is especially true for an Android Vulkan layer. When querying the capabilities of Vulkan
/// layers, the Android Vulkan loader will load the shared object and call into the exposed
/// introspection queries, and unload the shared object once the task is done. The Android Vulkan
/// loader may load the shared object later again if the layer is activated. However, on Android
/// there is no reliable way to register a callback when the shared object is actually unloaded.
///
/// Similar to [RefCell], even if `T` implements [Sync], [`LazyCollection<T>`] is not [Sync],
/// because a single-threaded way is used to test if the underlying collection is empty and destroy
/// the allocation
#[derive(Default)]
pub struct LazyCollection<T: CheckEmpty> {
    inner: OnceCell<T>,
    // Mark this type !Sync even if T implements Sync.
    marker: PhantomData<RefCell<T>>,
}

impl<T: CheckEmpty> LazyCollection<T> {
    /// Creates a new `LazyCollection` containing `value`.
    ///
    /// # Examples
    /// ```
    /// # use vulkan_layer::unstable_api::LazyCollection;
    /// let c = LazyCollection::new(vec![42]);
    /// ```
    #[allow(dead_code)]
    // Only test uses this function.
    pub fn new(value: T) -> Self {
        Self {
            inner: OnceCell::with_value(value),
            ..Default::default()
        }
    }

    /// Gets the reference to the underlying collection. Returns an owned empty T if the underlying
    /// collection is empty.
    ///
    /// # Examples
    /// ```
    /// # use vulkan_layer::unstable_api::LazyCollection;
    /// let vec = LazyCollection::new(vec![42]);
    /// let vec1 = vec.get();
    /// assert_eq!(*vec1, vec![42]);
    /// let vec2 = vec.get();
    /// // vec1 and vec2 point to the same location.
    /// assert!(std::ptr::eq(&*vec1, &*vec2));
    /// ```
    pub fn get(&self) -> Cow<'_, T> {
        // The destructor for None is a no-op, while this is not guaranteed for an empty T.
        // Therefore, we can't use &T as the return type and return a reference to a static empty T
        // when the underlying collection is empty.
        match self.inner.get() {
            Some(collection) => Cow::Borrowed(collection),
            None => Cow::Owned(Default::default()),
        }
    }

    /// Gets a mutable reference to the underlying collection, create an empty collection if the
    /// underlying collection was empty.
    ///
    /// # Examples
    /// ```
    /// # use vulkan_layer::unstable_api::LazyCollection;
    /// let mut vec = LazyCollection::<Vec<u32>>::default();
    /// let mut mut_vec = vec.get_mut_or_default();
    /// mut_vec.push(42);
    /// assert_eq!(*mut_vec, vec![42]);
    /// drop(mut_vec);
    ///
    /// let mut mut_vec = vec.get_mut_or_default();
    /// mut_vec.remove(0);
    /// assert_eq!(*mut_vec, vec![]);
    /// drop(mut_vec);
    /// // This won't cause a memory leak.
    /// std::mem::forget(vec);
    /// ```
    pub fn get_mut_or_default(&mut self) -> CollectionRefMut<'_, T> {
        // Ensure that inner is initialized.
        self.inner.get_or_init(Default::default);
        CollectionRefMut(&mut self.inner)
    }
}

/// A wrapper type for a mutably borrowed value from a [LazyCollection].
#[derive(Debug)]
pub struct CollectionRefMut<'a, T: CheckEmpty>(&'a mut OnceCell<T>);

impl<T: CheckEmpty> Drop for CollectionRefMut<'_, T> {
    fn drop(&mut self) {
        let should_destroy = self
            .0
            .get()
            .map(|collection| collection.is_empty())
            .unwrap_or(true);
        if !should_destroy {
            return;
        }
        self.0.take();
    }
}

impl<T: CheckEmpty> Deref for CollectionRefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // CollectionRefMut will always be initialized. get_mut_or_default is the only place we
        // initialize CollectionRefMut, and we never mutate it.
        self.0.get().unwrap()
    }
}

impl<T: CheckEmpty> DerefMut for CollectionRefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // CollectionRefMut will always be initialized. get_mut_or_default is the only place we
        // initialize CollectionRefMut, and we never mutate it.
        self.0.get_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_get_shouldnt_leak() {
        // We rely on the Miri test to detect the resource leak.
        let lazy_vec = LazyCollection::<Vec<u32>>::new(vec![]);
        let empty_vec = lazy_vec.get();
        assert!(empty_vec.is_empty());
        std::mem::forget(lazy_vec);

        let lazy_vec = LazyCollection::<Vec<u32>>::default();
        let empty_vec = lazy_vec.get();
        assert!(empty_vec.is_empty());
        std::mem::forget(lazy_vec);
    }

    #[test]
    fn test_non_empty_get_should_point_to_the_same_location() {
        let lazy_vec = LazyCollection::new(vec![42]);
        let vec1 = lazy_vec.get();
        assert_eq!(*vec1, vec![42]);
        let vec2 = lazy_vec.get();
        assert!(std::ptr::eq(&*vec1, &*vec2));
    }

    #[test]
    fn test_get_mut_should_get_the_content() {
        let mut lazy_vec = LazyCollection::new(vec![64]);
        {
            let mut vec = lazy_vec.get_mut_or_default();
            assert_eq!(*vec, vec![64]);
            vec.push(33);
            assert_eq!(*vec, vec![64, 33]);
        }
        let vec = lazy_vec.get_mut_or_default();
        assert_eq!(*vec, vec![64, 33]);
    }

    #[test]
    fn test_get_mut_empty_should_return_an_empty_collection() {
        let mut lazy_vec = LazyCollection::<Vec<u32>>::default();
        assert!(lazy_vec.get_mut_or_default().is_empty());
    }

    #[test]
    fn test_get_mut_empty_shouldnt_leak() {
        // We rely on the Miri test to detect the resource leak.
        let mut lazy_vec = LazyCollection::<Vec<u32>>::default();
        {
            let vec = lazy_vec.get_mut_or_default();
            assert!(vec.is_empty());
        }
        std::mem::forget(lazy_vec);
    }

    #[test]
    fn test_get_mut_insert_and_clear_shouldnt_leak() {
        // We rely on the Miri test to detect the resource leak.
        // 2 test cases. One on the same CollectionRefMut. One on 2 different CollectionRefMut's.
        {
            let mut lazy_vec = LazyCollection::<Vec<u32>>::default();
            let mut vec = lazy_vec.get_mut_or_default();
            vec.push(42);
            assert!(!vec.is_empty());
            vec.remove(0);
            assert!(vec.is_empty());
            drop(vec);
            std::mem::forget(lazy_vec);
        }
        {
            let mut lazy_vec = LazyCollection::<Vec<u32>>::default();
            let mut vec = lazy_vec.get_mut_or_default();
            vec.push(42);
            assert!(!vec.is_empty());
            drop(vec);
            let mut vec = lazy_vec.get_mut_or_default();
            vec.remove(0);
            assert!(vec.is_empty());
            drop(vec);
            std::mem::forget(lazy_vec);
        }
    }
}
