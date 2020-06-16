// use std::num::NonZeroU32;
//
// use ring::{
//     digest,
//     pbkdf2::{self, PBKDF2_HMAC_SHA1},
// };

// const KEY_LEN: usize = 64;
// const KEY_ITER: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(200_000) };

// pub fn init_key(secret: &str, salt: &[u8]) {
//     let mut output: [u8; KEY_LEN] = [0; KEY_LEN];
//     let k = pbkdf2::derive(
//         PBKDF2_HMAC_SHA1,
//         KEY_ITER,
//         salt,
//         secret.as_bytes(),
//         &mut output,
//     );
// }
