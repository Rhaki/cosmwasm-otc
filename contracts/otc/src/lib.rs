#[cfg(not(feature = "library"))]
pub mod contract;
mod execute;
mod query;
mod response;
mod state;
