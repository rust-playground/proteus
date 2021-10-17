//! Actions that impl the [Action](action/trait.Action.html) trait.

mod constant;
pub mod getter;
mod join;
mod len;
pub mod setter;
mod sum;

#[doc(inline)]
pub use constant::Constant;

#[doc(inline)]
pub use getter::Getter;

#[doc(inline)]
pub use join::Join;

#[doc(inline)]
pub use len::Len;

#[doc(inline)]
pub use sum::Sum;

#[doc(inline)]
pub use setter::Setter;
