//local shortcuts
use crate::*;

//third-party shortcuts
use bincode::Options;

//standard shortcuts
use core::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

//-------------------------------------------------------------------------------------------------------------------

//note: need to validate event ids against event registry to avoid memory use attack

//-------------------------------------------------------------------------------------------------------------------
