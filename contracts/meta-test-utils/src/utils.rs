use near_sdk::json_types::U128;

pub fn proportional(&self, amount: U128, numerator: U128, denominator: U128) -> U128 {
    U128::from(meta_tools::utils::proportional(amount.0, numerator.0, denominator.0))
}