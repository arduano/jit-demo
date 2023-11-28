// #![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::String;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Clone)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct User {
    pub email: String,
    pub gender: String,
    pub phone_number: String,
    pub birthdate: u64,
    pub location: Location,
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub title: String,
    pub picture: String,
}

#[derive(Clone)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct Location {
    pub street: String,
    pub city: String,
    pub state: String,
    pub postcode: u32,
}
