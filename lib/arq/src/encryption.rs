use ring::{digest, pbkdf2};

static CredentialAlgorithm: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
const CredentialSize: usize = digest::SHA256_OUTPUT_LEN;
pub type Credentials = [u8; CredentialSize];
const PBKDF2_ITERATIONS: usize = 2000;

struct CryptoCtx {}

// func bytesToKey(hf func() hash.Hash,
//                 salt, data []byte,
//                 iter int,
//                 keySize, ivSize int) (key, iv []byte) {
//  h := hf()
//  var d, dcat []byte
//  sum := make([]byte, 0, h.Size())
//  for len(dcat) < keySize+ivSize {
//    // D_i = HASH^count(D_(i-1) || data || salt)
//    h.Reset()
//    h.Write(d)
//    h.Write(data)
//    h.Write(salt)
//    sum = h.Sum(sum[:0])
//
//    for j := 1; j < iter; j++ {
//      h.Reset()
//      h.Write(sum)
//      sum = h.Sum(sum[:0])
//    }
//
//    d = append(d[:0], sum...)
//    dcat = append(dcat, d...)
//  }
//
//  return dcat[:keySize], dcat[keySize : keySize+ivSize]
//}

fn copy_digest(src: &Digest, dst: &mut [u8], digest_size: usize) {
    dst.copy_from_slice(&src.as_ref()[0..digest_size]);
}

struct AesKey {
    key: Vec<u8>,
    iv: Vec<u8>,
}

fn bytesToKey(
    input_key: &[u8],
    salt: &[u8],
    iterations: usize,
    key_len: usize,
    iv_len: usize,
) -> AesKey {
    let algorithm = &SHA1;
    let digest_len = algorithm.block_len;
    let bytes_required = key_len + iv_len;
    let mut output_buffer = Vec::new();
    let mut prev_digest = vec![0; digest_len];
    let mut sum = vec![0; digest_len];

    while output_buffer.len() < bytes_required {
        let mut hasher = Context::new(&SHA1);
        if output_buffer.is_empty() {
            hasher.update(prev_digest.as_slice());
        }
        hasher.update(input_key);
        hasher.update(salt);
        copy_digest(&hasher.finish(), &mut sum, digest_len);

        for _ in 1..iterations {
            copy_digest(&digest(&SHA1, &sum), &mut sum, digest_len);
        }

        prev_digest.copy_from_slice(&sum);
        output_buffer.append(&mut sum);
    }

    AesKey {
        key: output_buffer[0..key_len].to_vec(),
        iv: output_buffer[key_len..bytes_required].to_vec(),
    }
}

impl CryptoCtx {
    fn new(password: &str, salt: &[u8]) -> CryptoCtx {
        let mut key1 = [0; AES_KEY_LEN_BYTES + AES_IV_LEN_BYTES];
        pbkdf2::derive(
            SHA256,
            PBKDF2_ITERATIONS as u32,
            salt,
            password.as_bytes(),
            &mut key1,
        );

        // split the generated key into the AES key and initialisaton vector
        let AesKey { key, iv } = mk_aes_key(
            &key1,
            salt,
            PBKDF2_ITERATIONS,
            AES_KEY_LEN_BYTES,
            AES_IV_LEN_BYTES,
        );

        assert!(false);
        CryptoCtx {}
    }
}

#[cfg(test)]
mod test {}
