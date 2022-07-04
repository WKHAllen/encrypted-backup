use crate::crypto;
use crate::types::*;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

const LEN_SIZE: usize = 5;
const READER_CAPACITY: usize = 1 << 16; // 64 KB

/// Encode the size portion of a section of data.
///
/// `size`: the section size.
///
/// Returns the section size encoded in bytes.
fn encode_section_size(mut size: usize) -> [u8; LEN_SIZE] {
    let mut encoded_size = [0u8; LEN_SIZE];

    for i in 0..LEN_SIZE {
        encoded_size[LEN_SIZE - i - 1] = u8::try_from(size % 256).unwrap();
        size >>= 8;
    }

    encoded_size
}

/// Decode the size portion of a section of data.
///
/// `encoded_size`: the section size encoded in bytes.
///
/// Returns the size of the section.
fn decode_section_size(encoded_size: &[u8; LEN_SIZE]) -> usize {
    let mut size: usize = 0;

    for i in 0..LEN_SIZE {
        size <<= 8;
        size += usize::from(encoded_size[i]);
    }

    size
}

/// Encrypt a backup in chunks.
///
/// `tar_path`: the path to the tar archive.
/// `output_path`: the path to save the encrypted backup to.
/// `key`: the key to use when encrypting the backup.
///
/// Returns a result of the error variant if an error occurred while encrypting.
pub fn encrypt_backup(tar_path: &Path, output_path: &Path, key: &[u8; 32]) -> BackupResult<()> {
    let mut buffer = [0u8; READER_CAPACITY];
    let mut in_file = File::open(&tar_path)?;
    let mut out_file = File::create(&output_path)?;

    loop {
        let n = in_file.read(&mut buffer)?;

        if n == 0 {
            break;
        }

        let encrypted_data = crypto::aes_encrypt(&key, &buffer)?;
        let encoded_size = encode_section_size(encrypted_data.len());

        out_file.write_all(&encoded_size)?;
        out_file.write_all(&encrypted_data)?;
    }

    Ok(())
}

/// Decrypt a backup in chunks.
///
/// `path`: the path to the backup.
/// `key`: the key to use when decrypting the backup.
///
/// Returns a result containing the decrypted tar file, or the error variant if an error occurred while decrypting.
pub fn decrypt_backup(path: &Path, key: &[u8; 32]) -> BackupResult<File> {
    let mut size_buffer = [0u8; LEN_SIZE];
    let mut in_file = File::open(&path)?;
    let mut out_file = tempfile::tempfile()?;

    loop {
        let n = in_file.read(&mut size_buffer)?;

        if n == 0 {
            break;
        }

        let decoded_size = decode_section_size(&size_buffer);
        let mut buffer = vec![0u8; decoded_size];

        let n = in_file.read(&mut buffer)?;

        if n != decoded_size {
            return Err(BackupError::BackupReadFailed(
                "read fewer bytes from backup file than expected".to_owned(),
            ));
        }

        let decrypted_data = crypto::aes_decrypt(&key, &buffer)?;

        out_file.write_all(&decrypted_data)?;
    }

    out_file.seek(SeekFrom::Start(0))?;

    Ok(out_file)
}
