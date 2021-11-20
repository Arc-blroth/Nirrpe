//! # Nirrpe
//! An infinitely extensible RPG bookkeeper Discord bot!
//!
//! This is the core library for Nirrpe. It includes all of
//! Nirrpe's functionality except for the `main` function
//! and hot script reloading.
//! This allows both the dynamically generated script crates
//! and the main Nirrpe application to link to the same library.

#![feature(derive_default_enum)]

pub mod schema;

#[cfg(test)]
mod tests;
