extern crate bytes;

use crate::frame::{Body, Frame, Version, Writeable};
use crate::result::RSocketResult;

use bytes::{BigEndian, BufMut, ByteOrder, Bytes, BytesMut};

#[derive(Debug)]
pub struct Resume {
  version: Version,
  token: Option<Bytes>,
  last_received_server_position: u64,
  first_available_client_position: u64,
}

pub struct ResumeBuilder {
  stream_id: u32,
  flag: u16,
  inner: Resume,
}

impl Resume {
  fn new() -> Resume {
    Resume {
      version: Version::default(),
      token: None,
      last_received_server_position: 0,
      first_available_client_position: 0,
    }
  }

  pub fn decode(flag: u16, b: &mut BytesMut) -> RSocketResult<Resume> {
    let major = BigEndian::read_u16(b);
    b.advance(2);
    let minor = BigEndian::read_u16(b);
    b.advance(2);
    let token_size = BigEndian::read_u16(b);
    b.advance(2);
    let token = if token_size > 0 {
      Some(Bytes::from(b.split_to(token_size as usize)))
    } else {
      None
    };
    let p1 = BigEndian::read_u64(b);
    b.advance(8);
    let p2 = BigEndian::read_u64(b);
    b.advance(8);
    Ok(Resume {
      version: Version::new(major, minor),
      token: token,
      last_received_server_position: p1,
      first_available_client_position: p2,
    })
  }

  pub fn builder(stream_id: u32, flag: u16) -> ResumeBuilder {
    ResumeBuilder::new(stream_id, flag)
  }

  pub fn get_version(&self) -> Version {
    self.version.clone()
  }

  pub fn get_token(&self) -> Option<Bytes> {
    self.token.clone()
  }

  pub fn get_last_received_server_position(&self) -> u64 {
    self.last_received_server_position.clone()
  }

  pub fn get_first_available_client_position(&self) -> u64 {
    self.first_available_client_position.clone()
  }
}

impl ResumeBuilder {
  fn new(stream_id: u32, flag: u16) -> ResumeBuilder {
    ResumeBuilder {
      stream_id: stream_id,
      flag: flag,
      inner: Resume::new(),
    }
  }

  pub fn set_token(mut self, token: Bytes) -> Self {
    self.inner.token = Some(token);
    self
  }

  pub fn set_last_received_server_position(mut self, position: u64) -> Self {
    self.inner.last_received_server_position = position;
    self
  }

  pub fn set_first_available_client_position(mut self, position: u64) -> Self {
    self.inner.first_available_client_position = position;
    self
  }

  pub fn build(self) -> Frame {
    Frame {
      stream_id: self.stream_id,
      flag: self.flag,
      body: Body::Resume(self.inner),
    }
  }
}

impl Writeable for Resume {
  fn write_to(&self, bf: &mut BytesMut) {
    self.version.write_to(bf);
    if let Some(b) = &self.token {
      bf.put_u16_be(b.len() as u16);
      bf.put(b);
    }
    bf.put_u64_be(self.get_last_received_server_position());
    bf.put_u64_be(self.get_first_available_client_position());
  }

  fn len(&self) -> u32 {
    let mut size: u32 = 22;
    if let Some(b) = &self.token {
      size += b.len() as u32;
    }
    size
  }
}
