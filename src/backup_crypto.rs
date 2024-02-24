use crate::crypto::*;
use crate::types::*;
use std::fs::File as StdFile;
use std::io::{self, Read, Seek, Write};
use std::path::Path;
use tokio::fs::File as TokioFile;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

/// The length of the size portion of each chunk of data.
pub const LEN_SIZE: usize = 5;

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

    encoded_size.iter().for_each(|val| {
        size <<= 8;
        size += usize::from(*val);
    });

    size
}

/// Reads a section of data from a file synchronously.
fn read_section_sync(file: &mut StdFile) -> io::Result<Option<Vec<u8>>> {
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

/// Reads a section of data from a file asynchronously.
async fn read_section_async(file: &mut TokioFile) -> io::Result<Option<Vec<u8>>> {
    let mut size_buffer = [0u8; LEN_SIZE];

    let n = file.read(&mut size_buffer).await?;

    if n == 0 {
        return Ok(None);
    }

    let decoded_size = decode_section_size(&size_buffer);
    let mut buffer = vec![0u8; decoded_size];

    let n = file.read(&mut buffer).await?;

    if n != decoded_size {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "read fewer bytes from file than expected",
        ));
    }

    Ok(Some(buffer))
}

/// Writes a section of data to a file synchronously.
fn write_section_sync(file: &mut StdFile, data: &[u8]) -> io::Result<()> {
    let encoded_size = encode_section_size(data.len());

    file.write_all(&encoded_size)?;
    file.write_all(data)?;

    Ok(())
}

/// Writes a section of data to a file asynchronously.
async fn write_section_async(file: &mut TokioFile, data: &[u8]) -> io::Result<()> {
    let encoded_size = encode_section_size(data.len());

    file.write_all(&encoded_size).await?;
    file.write_all(data).await?;

    Ok(())
}

/// Encrypts a file in chunks synchronously.
fn encrypt_file_sync(
    src: &mut StdFile,
    dest: &mut StdFile,
    key: &[u8; AES_KEY_SIZE],
    chunk_size: usize,
) -> BackupResult<()> {
    let mut buffer = vec![0u8; chunk_size];

    loop {
        let n = src.read(&mut buffer)?;

        if n == 0 {
            break;
        }

        let encrypted_data = aes_encrypt(key, &buffer[..n])?;
        write_section_sync(dest, &encrypted_data)?;
    }

    dest.rewind()?;
    dest.flush()?;

    Ok(())
}

/// Encrypts a file in chunks asynchronously.
async fn encrypt_file_async(
    src: &mut TokioFile,
    dest: &mut TokioFile,
    key: &[u8; AES_KEY_SIZE],
    chunk_size: usize,
) -> BackupResult<()> {
    let mut buffer = vec![0u8; chunk_size];

    loop {
        let n = src.read(&mut buffer).await?;

        if n == 0 {
            break;
        }

        let encrypted_data = aes_encrypt(key, &buffer[..n])?;
        write_section_async(dest, &encrypted_data).await?;
    }

    dest.rewind().await?;
    dest.flush().await?;

    Ok(())
}

/// Decrypts a file in chunks synchronously.
fn decrypt_file_sync(
    src: &mut StdFile,
    dest: &mut StdFile,
    key: &[u8; AES_KEY_SIZE],
) -> BackupResult<()> {
    loop {
        let data = match read_section_sync(src)? {
            Some(data) => data,
            None => break,
        };

        let decrypted_data = aes_decrypt(key, &data)?;

        dest.write_all(&decrypted_data)?;
    }

    dest.rewind()?;
    dest.flush()?;

    Ok(())
}

/// Decrypts a file in chunks asynchronously.
async fn decrypt_file_async(
    src: &mut TokioFile,
    dest: &mut TokioFile,
    key: &[u8; AES_KEY_SIZE],
) -> BackupResult<()> {
    loop {
        let data = match read_section_async(src).await? {
            Some(data) => data,
            None => break,
        };

        let decrypted_data = aes_decrypt(key, &data)?;

        dest.write_all(&decrypted_data).await?;
    }

    dest.rewind().await?;
    dest.flush().await?;

    Ok(())
}

/// Encrypts a backup file in chunks synchronously.
pub fn encrypt_backup_sync(
    src_path: impl AsRef<Path>,
    dest_path: impl AsRef<Path>,
    key: &[u8; AES_KEY_SIZE],
    chunk_size: usize,
) -> BackupResult<()> {
    let mut src = StdFile::open(src_path)?;
    let mut dest = StdFile::create(dest_path)?;

    encrypt_file_sync(&mut src, &mut dest, key, chunk_size)
}

/// Encrypts a backup file in chunks asynchronously.
pub async fn encrypt_backup_async(
    src_path: impl AsRef<Path>,
    dest_path: impl AsRef<Path>,
    key: &[u8; AES_KEY_SIZE],
    chunk_size: usize,
) -> BackupResult<()> {
    let mut src = TokioFile::open(src_path).await?;
    let mut dest = TokioFile::create(dest_path).await?;

    encrypt_file_async(&mut src, &mut dest, key, chunk_size).await
}

/// Decrypts a backup file in chunks synchronously.
pub fn decrypt_backup_sync(
    src_path: impl AsRef<Path>,
    key: &[u8; AES_KEY_SIZE],
) -> BackupResult<StdFile> {
    let mut src = StdFile::open(src_path)?;
    let mut dest = tempfile::tempfile()?;

    decrypt_file_sync(&mut src, &mut dest, key)?;

    Ok(dest)
}

/// Decrypts a backup file in chunks asynchronously.
pub async fn decrypt_backup_async(
    src_path: impl AsRef<Path>,
    key: &[u8; AES_KEY_SIZE],
) -> BackupResult<TokioFile> {
    let mut src = TokioFile::open(src_path).await?;
    let mut dest = TokioFile::from_std(tempfile::tempfile()?);

    decrypt_file_async(&mut src, &mut dest, key).await?;

    Ok(dest)
}

/// Backup crypto tests.
#[cfg(test)]
mod tests {
    use super::*;
    use rand::{random, thread_rng, Fill};
    use tokio::time::Instant;

    fn rand_range(min: usize, max: usize) -> usize {
        (random::<usize>() % (max - min)) + min
    }

    fn encrypt_decrypt_file_sync(
        data: &[u8],
        password: &str,
        chunk_size: usize,
    ) -> (Vec<u8>, Vec<u8>) {
        let key = password_to_key(password);

        let mut plaintext_file = tempfile::tempfile().unwrap();
        plaintext_file.write_all(data).unwrap();
        plaintext_file.rewind().unwrap();

        let mut ciphertext_file = tempfile::tempfile().unwrap();
        encrypt_file_sync(&mut plaintext_file, &mut ciphertext_file, &key, chunk_size).unwrap();

        plaintext_file.rewind().unwrap();
        let mut plaintext_value = Vec::new();
        plaintext_file.read_to_end(&mut plaintext_value).unwrap();
        plaintext_file.rewind().unwrap();
        ciphertext_file.rewind().unwrap();
        let mut ciphertext_value = Vec::new();
        ciphertext_file.read_to_end(&mut ciphertext_value).unwrap();
        ciphertext_file.rewind().unwrap();

        let mut decrypted_file = tempfile::tempfile().unwrap();
        decrypt_file_sync(&mut ciphertext_file, &mut decrypted_file, &key).unwrap();

        decrypted_file.rewind().unwrap();
        let mut decrypted_value = Vec::new();
        decrypted_file.read_to_end(&mut decrypted_value).unwrap();
        decrypted_file.rewind().unwrap();

        (ciphertext_value, plaintext_value)
    }

    async fn encrypt_decrypt_file_async(
        data: &[u8],
        password: &str,
        chunk_size: usize,
    ) -> (Vec<u8>, Vec<u8>) {
        let key = password_to_key(password);

        let mut plaintext_file = TokioFile::from_std(tempfile::tempfile().unwrap());
        plaintext_file.write_all(data).await.unwrap();
        plaintext_file.rewind().await.unwrap();

        let mut ciphertext_file = TokioFile::from_std(tempfile::tempfile().unwrap());
        encrypt_file_async(&mut plaintext_file, &mut ciphertext_file, &key, chunk_size)
            .await
            .unwrap();

        plaintext_file.rewind().await.unwrap();
        let mut plaintext_value = Vec::new();
        plaintext_file
            .read_to_end(&mut plaintext_value)
            .await
            .unwrap();
        plaintext_file.rewind().await.unwrap();
        ciphertext_file.rewind().await.unwrap();
        let mut ciphertext_value = Vec::new();
        ciphertext_file
            .read_to_end(&mut ciphertext_value)
            .await
            .unwrap();
        ciphertext_file.rewind().await.unwrap();

        let mut decrypted_file = TokioFile::from_std(tempfile::tempfile().unwrap());
        decrypt_file_async(&mut ciphertext_file, &mut decrypted_file, &key)
            .await
            .unwrap();

        decrypted_file.rewind().await.unwrap();
        let mut decrypted_value = Vec::new();
        decrypted_file
            .read_to_end(&mut decrypted_value)
            .await
            .unwrap();
        decrypted_file.rewind().await.unwrap();

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
    fn test_file_encryption_sync() {
        let mut rng = thread_rng();

        let file_message = "Hello, encrypted file!";
        let password = "password123";
        let chunk_size = 1024;

        let (ciphertext, plaintext) =
            encrypt_decrypt_file_sync(file_message.as_bytes(), password, chunk_size);
        assert_ne!(&ciphertext, file_message.as_bytes());
        assert_eq!(&plaintext, file_message.as_bytes());
        assert_ne!(plaintext, ciphertext);

        let large_data_size = rand_range(1 << 19, 1 << 20);
        let mut large_data = vec![0u8; large_data_size];
        large_data.try_fill(&mut rng).unwrap();

        let (ciphertext, plaintext) = encrypt_decrypt_file_sync(&large_data, password, chunk_size);
        assert_ne!(ciphertext, large_data);
        assert_eq!(plaintext, large_data);
        assert_ne!(plaintext, ciphertext);
    }

    #[tokio::test]
    async fn test_file_encryption_async() {
        let mut rng = thread_rng();

        let file_message = "Hello, encrypted file!";
        let password = "password123";
        let chunk_size = 1024;

        let (ciphertext, plaintext) =
            encrypt_decrypt_file_async(file_message.as_bytes(), password, chunk_size).await;
        assert_ne!(&ciphertext, file_message.as_bytes());
        assert_eq!(&plaintext, file_message.as_bytes());
        assert_ne!(plaintext, ciphertext);

        let large_data_size = rand_range(1 << 19, 1 << 20);
        let mut large_data = vec![0u8; large_data_size];
        large_data.try_fill(&mut rng).unwrap();

        let (ciphertext, plaintext) =
            encrypt_decrypt_file_async(&large_data, password, chunk_size).await;
        assert_ne!(ciphertext, large_data);
        assert_eq!(plaintext, large_data);
        assert_ne!(plaintext, ciphertext);
    }

    #[tokio::test]
    async fn test_sync_async_speeds() {
        let mut rng = thread_rng();

        let password = "password123";
        let chunk_size = 1024;

        let large_data_size = rand_range(1 << 21, 1 << 22);
        let mut large_data = vec![0u8; large_data_size];
        large_data.try_fill(&mut rng).unwrap();

        let start = Instant::now();
        let (ciphertext, plaintext) = encrypt_decrypt_file_sync(&large_data, password, chunk_size);
        let end = Instant::now();
        assert_ne!(ciphertext, large_data);
        assert_eq!(plaintext, large_data);
        assert_ne!(plaintext, ciphertext);
        let delta = end.duration_since(start);
        println!(
            "Sync file encryption/decryption completed in {}s",
            (delta.as_micros() as f64) / 1_000_000f64
        );

        let start = Instant::now();
        let (ciphertext, plaintext) =
            encrypt_decrypt_file_async(&large_data, password, chunk_size).await;
        let end = Instant::now();
        assert_ne!(ciphertext, large_data);
        assert_eq!(plaintext, large_data);
        assert_ne!(plaintext, ciphertext);
        let delta = end.duration_since(start);
        println!(
            "Async file encryption/decryption completed in {}s",
            (delta.as_micros() as f64) / 1_000_000f64
        );
    }
}
