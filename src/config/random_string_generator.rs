use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum StringGenerator {
    /// Generates an alphanumeric string of the given length.
    Random {
        length: usize,
        /// How many collisions to retry before bumping `length` by 1.
        #[serde(default = "StringGenerator::default_max_attempts_before_grow")]
        max_attempts_before_grow: u32,
    },

    /// Generates a new v4 UUID.
    Uuid,
}

impl StringGenerator {
    /// The serde default for `max_attempts_before_grow`
    fn default_max_attempts_before_grow() -> u32 {
        32
    }

    pub fn generate(&self) -> String {
        match *self {
            StringGenerator::Random {
                length,
                max_attempts_before_grow: _,
            } => rand::rng()
                .sample_iter(&Alphanumeric)
                .take(length)
                .map(char::from)
                .collect(),
            StringGenerator::Uuid => Uuid::new_v4().to_string(),
        }
    }

    /// Expose the max‐attempts value to callers.
    pub fn max_attempts_before_grow(&self) -> &u32 {
        match self {
            StringGenerator::Random {
                length: _,
                max_attempts_before_grow,
            } => max_attempts_before_grow,
            StringGenerator::Uuid => &0,
        }
    }
}

impl Default for StringGenerator {
    fn default() -> Self {
        StringGenerator::Random {
            length: 12,
            max_attempts_before_grow:
                StringGenerator::default_max_attempts_before_grow(),
        }
    }
}
