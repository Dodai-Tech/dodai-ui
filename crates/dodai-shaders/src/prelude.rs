//! Float traits the GPU build needs (`no_std` has no inherent float methods).
//! Empty on the CPU.
#[cfg(target_arch = "spirv")]
pub use spirv_std::num_traits::Float;
