use crate::crypto::*;
use crate::types::*;
use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::path::Path;

/// The length of the size portion of each chunk of data.
pub const LEN_SIZE: usize = 5;

/// The maximum size of data to read at once.
pub const READER_CAPACITY: usize = 1 << 10; // 1 KiB

/// Encodes the size portion of a section of data.
pub fn encode_section_size(mut size: usize) -> [u8; LEN_SIZE] {
    let mut encoded_size = [0u8; LEN_SIZE];

    for i in 0..LEN_SIZE {
        encoded_size[LEN_SIZE - i - 1] = u8::try_from(size % 256).unwrap();
        size >>= 8;
    }

    encoded_size
}

/// Decodes the size portion of a section of data.
pub fn decode_section_size(encoded_size: &[u8; LEN_SIZE]) -> usize {
    let mut size: usize = 0;

    for i in 0..LEN_SIZE {
        size <<= 8;
        size += usize::from(encoded_size[i]);
    }

    size
}

/// Reads a section of data from a file.
fn read_section(file: &mut File) -> io::Result<Option<Vec<u8>>> {
    let mut size_buffer = [0u8; LEN_SIZE];

    let n = file.read(&mut size_buffer)?;

    if n == 0 {
        return Ok(None);
    }

    let decoded_size = decode_section_size(&size_buffer);
    let mut buffer = vec![0u8; decoded_size];

    let n = file.read(&mut buffer)?;

    if n != decoded_size {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "read fewer bytes from file than expected",
        ));
    }

    Ok(Some(buffer))
}

/// Writes a section of data to a file.
fn write_section(file: &mut File, data: &[u8]) -> io::Result<()> {
    let encoded_size = encode_section_size(data.len());

    file.write_all(&encoded_size)?;
    file.write_all(&data)?;

    Ok(())
}

/// Encrypts a file in chunks.
fn encrypt_file(src: &mut File, dest: &mut File, key: &[u8; AES_KEY_SIZE]) -> BackupResult<()> {
    let mut buffer = [0u8; READER_CAPACITY];

    loop {
        let n = src.read(&mut buffer)?;

        if n == 0 {
            break;
        }

        let encrypted_data = aes_encrypt(&key, &buffer[..n])?;
        write_section(dest, &encrypted_data)?;
    }

    dest.rewind()?;
    dest.flush()?;

    Ok(())
}

/// Decrypts a file in chunks.
fn decrypt_file(src: &mut File, dest: &mut File, key: &[u8; AES_KEY_SIZE]) -> BackupResult<()> {
    loop {
        let data = match read_section(src)? {
            Some(data) => data,
            None => break,
        };

        let decrypted_data = aes_decrypt(&key, &data)?;

        dest.write_all(&decrypted_data)?;
    }

    dest.rewind()?;
    dest.flush()?;

    Ok(())
}

/// Encrypts a backup file in chunks.
pub fn encrypt_backup(
    src_path: impl AsRef<Path>,
    dest_path: impl AsRef<Path>,
    key: &[u8; AES_KEY_SIZE],
) -> BackupResult<()> {
    let mut src = File::open(src_path)?;
    let mut dest = File::create(dest_path)?;

    encrypt_file(&mut src, &mut dest, key)
}

/// Decrypts a backup file in chunks.
pub fn decrypt_backup(src_path: impl AsRef<Path>, key: &[u8; AES_KEY_SIZE]) -> BackupResult<File> {
    let mut src = File::open(src_path)?;
    let mut dest = tempfile::tempfile()?;

    decrypt_file(&mut src, &mut dest, key)?;

    Ok(dest)
}

/// Backup crypto tests.
#[cfg(test)]
mod tests {
    use super::*;
    use rand::{random, thread_rng, Fill};

    fn rand_range(min: usize, max: usize) -> usize {
        (random::<usize>() % (max - min)) + min
    }

    fn encrypt_decrypt_file(data: &[u8], password: &str) -> (Vec<u8>, Vec<u8>) {
        let key = password_to_key(password);

        let mut plaintext_file = tempfile::tempfile().unwrap();
        plaintext_file.write_all(data).unwrap();
        plaintext_file.rewind().unwrap();

        let mut ciphertext_file = tempfile::tempfile().unwrap();
        encrypt_file(&mut plaintext_file, &mut ciphertext_file, &key).unwrap();

        plaintext_file.rewind().unwrap();
        let mut plaintext_value = Vec::new();
        plaintext_file.read_to_end(&mut plaintext_value).unwrap();
        plaintext_file.rewind().unwrap();
        ciphertext_file.rewind().unwrap();
        let mut ciphertext_value = Vec::new();
        ciphertext_file.read_to_end(&mut ciphertext_value).unwrap();
        ciphertext_file.rewind().unwrap();

        let mut decrypted_file = tempfile::tempfile().unwrap();
        decrypt_file(&mut ciphertext_file, &mut decrypted_file, &key).unwrap();

        decrypted_file.rewind().unwrap();
        let mut decrypted_value = Vec::new();
        decrypted_file.read_to_end(&mut decrypted_value).unwrap();
        decrypted_file.rewind().unwrap();

        (ciphertext_value, plaintext_value)
    }

    #[test]
    fn test_encode_section_size() {
        assert_eq!(encode_section_size(0), [0, 0, 0, 0, 0]);
        assert_eq!(encode_section_size(1), [0, 0, 0, 0, 1]);
        assert_eq!(encode_section_size(255), [0, 0, 0, 0, 255]);
        assert_eq!(encode_section_size(256), [0, 0, 0, 1, 0]);
        assert_eq!(encode_section_size(257), [0, 0, 0, 1, 1]);
        assert_eq!(encode_section_size(4311810305), [1, 1, 1, 1, 1]);
        assert_eq!(encode_section_size(4328719365), [1, 2, 3, 4, 5]);
        assert_eq!(encode_section_size(47362409218), [11, 7, 5, 3, 2]);
        assert_eq!(
            encode_section_size(1099511627775),
            [255, 255, 255, 255, 255]
        );
    }

    #[test]
    fn test_decode_section_size() {
        assert_eq!(decode_section_size(&[0, 0, 0, 0, 0]), 0);
        assert_eq!(decode_section_size(&[0, 0, 0, 0, 1]), 1);
        assert_eq!(decode_section_size(&[0, 0, 0, 0, 255]), 255);
        assert_eq!(decode_section_size(&[0, 0, 0, 1, 0]), 256);
        assert_eq!(decode_section_size(&[0, 0, 0, 1, 1]), 257);
        assert_eq!(decode_section_size(&[1, 1, 1, 1, 1]), 4311810305);
        assert_eq!(decode_section_size(&[1, 2, 3, 4, 5]), 4328719365);
        assert_eq!(decode_section_size(&[11, 7, 5, 3, 2]), 47362409218);
        assert_eq!(
            decode_section_size(&[255, 255, 255, 255, 255]),
            1099511627775
        );
    }

    #[test]
    fn test_file_encryption() {
        let mut rng = thread_rng();

        let file_message = "Hello, encrypted file!";
        let password = "password123";

        let (ciphertext, plaintext) = encrypt_decrypt_file(file_message.as_bytes(), password);
        assert_ne!(&ciphertext, file_message.as_bytes());
        assert_eq!(&plaintext, file_message.as_bytes());
        assert_ne!(plaintext, ciphertext);

        let large_data_size = rand_range(1 << 19, 1 << 20);
        let mut large_data = vec![0u8; large_data_size];
        large_data.try_fill(&mut rng).unwrap();

        let (ciphertext, plaintext) = encrypt_decrypt_file(&large_data, password);
        assert_ne!(ciphertext, large_data);
        assert_eq!(plaintext, large_data);
        assert_ne!(plaintext, ciphertext);
    }
}
