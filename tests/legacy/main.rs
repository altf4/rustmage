//! Existing assurance suite, compiled separately from the engine.
#![allow(unused_imports, dead_code)]
use rustmage::*;
use rustmage_core::capacity;
use rustmage_core::card_names;
use rustmage_core::cards;
use rustmage_core::permanent_type;
use rustmage_core::shared_vec;
mod engine;
use rustmage_core::rng;

#[path = "../support/mod.rs"]
mod support;
