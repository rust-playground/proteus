//! Actions that impl the [Action](action/trait.Action.html) trait.

mod constant;
pub mod getter;
mod join;
pub mod setter;

#[doc(inline)]
pub use constant::Constant;

#[doc(inline)]
pub use getter::Getter;

#[doc(inline)]
pub use join::Join;

#[doc(inline)]
pub use setter::Setter;
