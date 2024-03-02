//! Utilities for applying cryptography to a backup.

use crate::crypto::*;
use crate::pool::*;
use crate::types::*;
use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::path::Path;
use std::thread::scope;

/// The length of the size portion of each chunk of data.
pub const LEN_SIZE: usize = 5;

/// Encodes the size portion of a section of data.
pub fn encode_section_size(size: usize) -> [u8; LEN_SIZE] {
    (0..LEN_SIZE)
        .fold((size, [0u8; LEN_SIZE]), |(size, mut encoded_size), i| {
            encoded_size[LEN_SIZE - i - 1] = u8::try_from(size % 256).unwrap();
            (size >> 8, encoded_size)
        })
        .1
}

/// Decodes the size portion of a section of data.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn decode_section_size(encoded_size: &[u8; LEN_SIZE]) -> usize {
    encoded_size
        .iter()
        .fold(0, |size, val| (size << 8) + usize::from(*val))
}

/// Gets the chunk size of a given backup file.
pub fn get_chunk_size(path: impl AsRef<Path>) -> io::Result<usize> {
    let mut file = File::open(path)?;
    let mut size_buffer = [0u8; LEN_SIZE];

    let n = file.read(&mut size_buffer)?;

    if n != LEN_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "not enough bytes to read from backup file",
        ));
    }

    Ok(decode_section_size(&size_buffer))
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
    file.write_all(data)?;

    Ok(())
}

/// Encrypts a file in chunks.
fn encrypt_file(
    src: &mut File,
    dest: &mut File,
    key: [u8; AES_KEY_SIZE],
    chunk_size: usize,
    pool_size: u8,
) -> BackupResult<()> {
    let (task_request, task_response) = task_channel(pool_size.into());

    scope(|s| {
        let read_handle = s.spawn(move || {
            loop {
                let mut buffer = vec![0u8; chunk_size];

                let n = src.read(&mut buffer)?;

                if n == 0 {
                    break;
                }

                if task_request
                    .send(move || aes_encrypt(key, &buffer[..n]))
                    .is_err()
                {
                    // The receiver has closed prematurely, meaning it most
                    // likely encountered an error.
                    break;
                }
            }

            BackupResult::Ok(())
        });

        let write_handle = s.spawn(|| {
            // `task_response` must be explicitly moved into this closure, but
            // the closure cannot be a move closure, as that will move `dest`
            // into it, which will cause problems below.
            let task_response = task_response;

            while let Some(encrypted_data) = task_response.recv() {
                write_section(dest, &encrypted_data?)?;
            }

            BackupResult::Ok(())
        });

        read_handle.join().unwrap()?;
        write_handle.join().unwrap()?;
        BackupResult::Ok(())
    })?;

    dest.rewind()?;
    dest.flush()?;

    Ok(())
}

/// Decrypts a file in chunks.
fn decrypt_file(
    src: &mut File,
    dest: &mut File,
    key: [u8; AES_KEY_SIZE],
    pool_size: u8,
) -> BackupResult<()> {
    let (task_request, task_response) = task_channel(pool_size.into());

    scope(|s| {
        let read_handle = s.spawn(move || {
            loop {
                let Some(data) = read_section(src)? else {
                    break;
                };

                if task_request.send(move || aes_decrypt(key, &data)).is_err() {
                    // The receiver has closed prematurely, meaning it most
                    // likely encountered an error.
                    break;
                }
            }

            BackupResult::Ok(())
        });

        let write_handle = s.spawn(|| {
            // `task_response` must be explicitly moved into this closure, but
            // the closure cannot be a move closure, as that will move `dest`
            // into it, which will cause problems below.
            let task_response = task_response;

            while let Some(decrypted_data) = task_response.recv() {
                dest.write_all(&decrypted_data?)?;
            }

            BackupResult::Ok(())
        });

        read_handle.join().unwrap()?;
        write_handle.join().unwrap()?;
        BackupResult::Ok(())
    })?;

    dest.rewind()?;
    dest.flush()?;

    Ok(())
}

/// Encrypts a backup file in chunks.
pub fn encrypt_backup(
    src_path: impl AsRef<Path>,
    dest_path: impl AsRef<Path>,
    key: [u8; AES_KEY_SIZE],
    chunk_size: usize,
    pool_size: u8,
) -> BackupResult<()> {
    let mut src = File::open(src_path)?;
    let mut dest = File::create(dest_path)?;

    encrypt_file(&mut src, &mut dest, key, chunk_size, pool_size)
}

/// Decrypts a backup file in chunks.
pub fn decrypt_backup(
    src_path: impl AsRef<Path>,
    key: [u8; AES_KEY_SIZE],
    pool_size: u8,
) -> BackupResult<File> {
    let mut src = File::open(src_path)?;
    let mut dest = tempfile::tempfile()?;

    decrypt_file(&mut src, &mut dest, key, pool_size)?;

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

    fn encrypt_decrypt_file(
        data: &[u8],
        password: &str,
        chunk_size: usize,
        pool_size: u8,
    ) -> (Vec<u8>, Vec<u8>) {
        let key = password_to_key(password);

        let mut plaintext_file = tempfile::tempfile().unwrap();
        plaintext_file.write_all(data).unwrap();
        plaintext_file.rewind().unwrap();

        let mut ciphertext_file = tempfile::tempfile().unwrap();
        encrypt_file(
            &mut plaintext_file,
            &mut ciphertext_file,
            key,
            chunk_size,
            pool_size,
        )
        .unwrap();

        plaintext_file.rewind().unwrap();
        let mut plaintext_value = Vec::new();
        plaintext_file.read_to_end(&mut plaintext_value).unwrap();
        plaintext_file.rewind().unwrap();
        ciphertext_file.rewind().unwrap();
        let mut ciphertext_value = Vec::new();
        ciphertext_file.read_to_end(&mut ciphertext_value).unwrap();
        ciphertext_file.rewind().unwrap();

        let mut decrypted_file = tempfile::tempfile().unwrap();
        decrypt_file(&mut ciphertext_file, &mut decrypted_file, key, pool_size).unwrap();

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
        assert_eq!(encode_section_size(4_311_810_305), [1, 1, 1, 1, 1]);
        assert_eq!(encode_section_size(4_328_719_365), [1, 2, 3, 4, 5]);
        assert_eq!(encode_section_size(47_362_409_218), [11, 7, 5, 3, 2]);
        assert_eq!(
            encode_section_size(1_099_511_627_775),
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
        assert_eq!(decode_section_size(&[1, 1, 1, 1, 1]), 4_311_810_305);
        assert_eq!(decode_section_size(&[1, 2, 3, 4, 5]), 4_328_719_365);
        assert_eq!(decode_section_size(&[11, 7, 5, 3, 2]), 47_362_409_218);
        assert_eq!(
            decode_section_size(&[255, 255, 255, 255, 255]),
            1_099_511_627_775
        );
    }

    #[test]
    fn test_file_encryption() {
        let mut rng = thread_rng();

        let file_message = "Hello, encrypted file!";
        let password = "password123";
        let chunk_size = 1 << 10;
        let pool_size = 16;

        let (ciphertext, plaintext) =
            encrypt_decrypt_file(file_message.as_bytes(), password, chunk_size, pool_size);
        assert_ne!(&ciphertext, file_message.as_bytes());
        assert_eq!(&plaintext, file_message.as_bytes());
        assert_ne!(plaintext, ciphertext);

        let large_data_size = rand_range(1 << 19, 1 << 20);
        let mut large_data = vec![0u8; large_data_size];
        large_data.try_fill(&mut rng).unwrap();

        let (ciphertext, plaintext) =
            encrypt_decrypt_file(&large_data, password, chunk_size, pool_size);
        assert_ne!(ciphertext, large_data);
        assert_eq!(plaintext, large_data);
        assert_ne!(plaintext, ciphertext);
    }
}
