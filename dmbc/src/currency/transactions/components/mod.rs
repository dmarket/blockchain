//! Common transaction components.

mod fees;
mod intermediary;

pub use currency::transactions::components::fees::{FeeStrategy, ThirdPartyFees, FeesCalculator};
pub use currency::transactions::components::intermediary::Intermediary;