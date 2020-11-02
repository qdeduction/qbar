// file: src/database.rs
// authors: Brandon H. Gomes

//! Expression Database Module

use core::borrow::Borrow;

/// Database key which generates a default value.
pub trait DatabaseKey<V> {
    /// Generate a key from the value.
    fn from_value(value: &V) -> Self;
}

/// Database entry which represents a key value pair with possibly generated keys.
pub trait DatabaseEntry<K, V>
where
    K: DatabaseKey<V>,
{
    /// Get the database key.
    fn key(&self) -> &K;

    /// Get the database value.
    fn value(&self) -> &V;

    /// Generate a new entry from a key and a value.
    fn new(key: K, value: V) -> Self;

    /// Generate a new entry from a value using the generated key.
    fn from_value(value: V) -> Self
    where
        Self: Sized,
    {
        Self::new(K::from_value(&value), value)
    }
}

/// Database Trait
pub trait Database<K, V>
where
    K: DatabaseKey<V>,
{
    /// Returns a reference to the value corresponding to the key.
    fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        Q: ?Sized + Borrow<K>;

    /// Returns a mutable reference to the value corresponding to the key.
    fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        Q: ?Sized + Borrow<K>;

    /// Returns `true` if the database contains a value for the specified key.
    fn contains_key<Q>(&self, k: &Q) -> bool
    where
        Q: ?Sized + Borrow<K>;

    /// Returns `true` if the database contains the value.
    fn contains_value(&self, v: &V) -> bool {
        self.contains_key(&K::from_value(v))
    }

    /// Inserts a key-value pair into the database.
    fn insert(&mut self, k: K, v: V) -> Option<K>;

    /// Inserts a value into the database using the generated key.
    fn insert_value(&mut self, v: V) -> Option<K>;

    /// Removes a key from the database, returning the value at the key if the
    /// key was previously in the map.
    fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        Q: ?Sized + Borrow<K>;

    /// Removes a value from the database using the generated key, returning
    /// the value if the value was previously in the database.
    fn remove_value(&mut self, v: &V) -> Option<V> {
        self.remove(&K::from_value(v))
    }

    /// Retains only the elements specified by the predicate.
    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool;

    /// Get the length of the database.
    fn len(&self) -> usize;

    /// Check if the database is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the database.
    fn clear(&mut self);
}

