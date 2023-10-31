
use crate::types::U256;
use near_sdk::{env, CryptoHash};

/*************************/
/*   Cryptographic Ops   */
/*************************/

#[inline]
pub fn generate_hash_id(id: String) -> CryptoHash {
    env::keccak256(id.as_bytes()).try_into().unwrap()
}

/****************/
/*   Math Ops   */
/****************/

#[inline]
/// returns amount * numerator/denominator
pub fn proportional(amount: u128, numerator: u128, denominator: u128) -> u128 {
    return (U256::from(amount) * U256::from(numerator) / U256::from(denominator)).as_u128();
}