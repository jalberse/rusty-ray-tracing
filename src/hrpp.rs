//! Hash-based ray path prediction.
//! Based on a technique proposed by Francois Demoullin, Ayub Gubran, Tor Aamodt
//! See https://arxiv.org/abs/1910.01304
//! Hash-Based Ray Path Prediction: Skipping BVH Traversal Computation by Exploiting Ray Locality

use ahash::AHashMap;

use crate::ray::Ray;

/// The number of bits extracted from float values'
/// exponent and mantissa. So the total number of bits
/// will be 2n + 1 (one extra being the sign bit).
/// The original paper found 5 bits to be optimal.
#[allow(dead_code)]
enum BitPrecision {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

// We define a predictor rather than using a has map directly because
// 1. The predictor can convert Ray to a u64 for use as a key in the hash map.
//    This is simpler than implementing Hash/Hasher for a Ray and using Ray as a key
//    directly, since our hashing technique is non-typical.
//    This matches the original paper's implementation which used a u64 as a key.
// 2. It provides a limited interface for predictions, which makes use simpler,
// 3. We could theoretically have the predictor be non-hash-based in the future.
//    This is a tertiary concern, though, really it's just simpler.
pub struct Predictor {
    // Maps the result of hash(ray) to the index of the predicted node for that hash.
    prediction_table: AHashMap<u64, usize>,
}

impl Predictor {
    pub fn new() -> Predictor {
        let prediction_table = AHashMap::new();
        Predictor { prediction_table }
    }

    /// Returns the prediction if there is one.
    /// If there is no prediction for this ray, returns None.
    pub fn get_prediction(&self, ray: &Ray) -> Option<&usize> {
        let key = hash(ray);
        self.prediction_table.get(&key)
    }

    pub fn has_prediction(&self, ray: &Ray) -> bool {
        let key = hash(ray);
        self.prediction_table.contains_key(&key)
    }

    /// See https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.insert
    pub fn insert(&mut self, ray: &Ray, prediction: usize) -> Option<usize> {
        let key = hash(ray);
        self.prediction_table.insert(key, prediction)
    }
}

fn map_float_to_hash(val: f32, bit_precision: &BitPrecision) -> u16 {
    let bits: u32 = val.to_bits();

    let sign_bit: u16 = (bits >> 31) as u16 & 0x1;

    let exponent_bits_to_shift: u16 = match bit_precision {
        BitPrecision::One => 30,
        BitPrecision::Two => 29,
        BitPrecision::Three => 28,
        BitPrecision::Four => 27,
        BitPrecision::Five => 26,
        BitPrecision::Six => 25,
        BitPrecision::Seven => 24,
    };

    let bits_to_mask: u16 = match bit_precision {
        BitPrecision::One => 0x1,
        BitPrecision::Two => 0x3,
        BitPrecision::Three => 0x7,
        BitPrecision::Four => 0xf,
        BitPrecision::Five => 0x1f,
        BitPrecision::Six => 0x3f,
        BitPrecision::Seven => 0x7f,
    };

    let exponent_bits: u16 = (bits >> exponent_bits_to_shift) as u16 & bits_to_mask;

    let mantissa_bits_to_shift: u16 = match bit_precision {
        BitPrecision::One => 22,
        BitPrecision::Two => 21,
        BitPrecision::Three => 20,
        BitPrecision::Four => 19,
        BitPrecision::Five => 18,
        BitPrecision::Six => 17,
        BitPrecision::Seven => 16,
    };

    let mantissa_bits: u16 = (bits >> mantissa_bits_to_shift) as u16 & bits_to_mask;

    sign_bit << 15 | exponent_bits << 7 | mantissa_bits
}

pub fn hash(ray: &Ray) -> u64 {
    // The original paper specifies that 5 is optimal, so we'll
    // hardcode that for now. We may expand to let this be configurable.
    let precision = BitPrecision::Five;

    let hash_origin_x = map_float_to_hash(ray.origin.x, &precision) as u64;
    let hash_origin_y = map_float_to_hash(ray.origin.y, &precision) as u64;
    let hash_origin_z = map_float_to_hash(ray.origin.z, &precision) as u64;
    let hash_direction_x = map_float_to_hash(ray.direction.x, &precision) as u64;
    let hash_direction_y = map_float_to_hash(ray.direction.y, &precision) as u64;
    let hsah_direction_z = map_float_to_hash(ray.direction.z, &precision) as u64;

    // xor the hashes to save space
    let hash_0 = hash_origin_x ^ hsah_direction_z;
    let hash_1 = hash_origin_y ^ hash_direction_y;
    let hash_2 = hash_origin_z ^ hash_direction_x;

    let predictor_table_index: u64 = (hash_0 << 0) | (hash_1 << 16) | (hash_2 << 32);

    predictor_table_index
}
