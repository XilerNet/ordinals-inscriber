extern crate crypto;

use crypto::buffer::{BufferResult, ReadBuffer, WriteBuffer};
use crypto::{aes, blockmodes, buffer};
use lazy_static::lazy_static;
use std::{env, str::from_utf8, sync::Arc};
use tracing::info;

use crate::db::encryption_methods::EncryptionMethods;

lazy_static! {
  static ref ENCRYPTION_KEY: Arc<Vec<u8>> = {
    let key_string =
      env::var("DATABASE_KEY").expect("DATABASE_KEY not set or is not 32 bytes long");

    let key = key_string.bytes().collect::<Vec<u8>>();
    info!("Encryption key loaded");

    Arc::new(key)
  };
}

fn pkcs7_padding(input: &[u8], block_size: usize) -> Vec<u8> {
  let padding_length = block_size - (input.len() % block_size);
  let mut padded = input.to_vec();
  padded.extend(vec![padding_length as u8; padding_length]);
  padded
}

fn pkcs7_unpadding(input: &[u8]) -> Vec<u8> {
  if let Some(&last_byte) = input.last() {
    if last_byte as usize <= input.len() {
      let padding_len = last_byte as usize;
      if input[input.len() - padding_len..input.len() - 1]
        .iter()
        .all(|&x| x == last_byte)
      {
        return input[..input.len() - padding_len].to_vec();
      }
    }
  }
  input.to_vec()
}

fn encrypt(plaintext: &[u8], encryption_method: EncryptionMethods) -> Vec<u8> {
  match encryption_method {
    EncryptionMethods::AES256 => {
      let padded_plaintext = pkcs7_padding(plaintext, 16); // Pad the plaintext to be multiple of 16 bytes
      let mut encryptor = aes::ecb_encryptor(
        aes::KeySize::KeySize256,
        &ENCRYPTION_KEY,
        blockmodes::NoPadding,
      );
      let mut final_result = Vec::<u8>::new();
      let mut read_buffer = buffer::RefReadBuffer::new(&padded_plaintext);
      let mut buffer = [0; 4096];
      let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

      loop {
        let result = encryptor
          .encrypt(&mut read_buffer, &mut write_buffer, true)
          .unwrap();

        final_result.extend(
          write_buffer
            .take_read_buffer()
            .take_remaining()
            .iter()
            .copied(),
        );

        match result {
          BufferResult::BufferUnderflow => break,
          BufferResult::BufferOverflow => {}
        }
      }

      final_result
    }
  }
}

fn decrypt(ciphertext: &[u8], encryption_method: EncryptionMethods) -> Vec<u8> {
  match encryption_method {
    EncryptionMethods::AES256 => {
      let mut decryptor = aes::ecb_decryptor(
        aes::KeySize::KeySize256,
        &ENCRYPTION_KEY,
        blockmodes::NoPadding,
      );
      let mut final_result = Vec::<u8>::new();
      let mut read_buffer = buffer::RefReadBuffer::new(ciphertext);
      let mut buffer = [0; 4096];
      let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

      loop {
        let result = decryptor
          .decrypt(&mut read_buffer, &mut write_buffer, true)
          .unwrap();

        final_result.extend(
          write_buffer
            .take_read_buffer()
            .take_remaining()
            .iter()
            .copied(),
        );

        match result {
          BufferResult::BufferUnderflow => break,
          BufferResult::BufferOverflow => {}
        }
      }

      pkcs7_unpadding(&final_result)
    }
  }
}

fn encrypt_as_string(content: &[u8], encryption_method: EncryptionMethods) -> String {
  let encrypted = encrypt(content, encryption_method);
  hex::encode(encrypted)
}

fn decrypt_as_string(content: &[u8], encryption_method: EncryptionMethods) -> String {
  let decoded = hex::decode(content).unwrap();
  let decrypted = decrypt(&decoded, encryption_method);
  from_utf8(&decrypted).unwrap().to_string()
}

pub fn encrypt_string(content: &str) -> (String, EncryptionMethods) {
  (
    encrypt_as_string(content.as_bytes(), EncryptionMethods::AES256),
    EncryptionMethods::AES256,
  )
}

pub fn encrypt_many<const LENGTH: usize>(
  content: [&str; LENGTH],
) -> ([String; LENGTH], EncryptionMethods) {
  (
    content.map(|x| encrypt_as_string(x.as_bytes(), EncryptionMethods::AES256)),
    EncryptionMethods::AES256,
  )
}

pub fn encrypt_many_vec(content: Vec<&str>) -> (Vec<String>, EncryptionMethods) {
  (
    content
      .into_iter()
      .map(|x| encrypt_as_string(x.as_bytes(), EncryptionMethods::AES256))
      .collect(),
    EncryptionMethods::AES256,
  )
}

pub fn decrypt_string(content: &str, encryption_method: EncryptionMethods) -> String {
  decrypt_as_string(content.as_bytes(), encryption_method)
}

pub fn decrypt_many<const LENGTH: usize>(
  content: [&str; LENGTH],
  encryption_method: EncryptionMethods,
) -> [String; LENGTH] {
  content.map(|x| decrypt_as_string(x.as_bytes(), encryption_method))
}
