use smol::io::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _};

use crate::prelude::*;

pub trait TheCurseReadWriteExt: Serialize + for<'de> Deserialize<'de> {
    #[inline(always)]
    fn write_to<S>(&self, stream: &mut S) -> impl Future<Output = Result<(), TheCurseIoError>>
    where
        S: AsyncWrite + Unpin,
    {
        async {
            let bytes = postcard::to_allocvec(self).map_err(TheCurseIoError::Serialization)?;
            stream
                .write_all(&(bytes.len() as u32).to_le_bytes())
                .await?;
            stream.write_all(&bytes).await?;
            stream.flush().await?;

            Ok(())
        }
    }

    #[inline(always)]
    fn read_from<S>(
        stream: &mut S,
        len_buf: &mut [u8; 4],
        buf: &mut Vec<u8>,
    ) -> impl Future<Output = Result<Self, TheCurseIoError>>
    where
        S: AsyncRead + Unpin,
        Self: Sized,
    {
        async {
            stream.read_exact(len_buf).await?;
            Self::read_from_with_len(stream, len_buf, buf).await
        }
    }

    #[inline(always)]
    /// Read online the body of the message from the stream, the len_buf should be manually read
    /// before.
    fn read_from_with_len<S>(
        stream: &mut S,
        len_buf: &[u8; 4],
        buf: &mut Vec<u8>,
    ) -> impl Future<Output = Result<Self, TheCurseIoError>>
    where
        S: AsyncRead + Unpin,
        Self: Sized,
    {
        async {
            let len = u32::from_le_bytes(*len_buf) as usize;
            if buf.len() < len {
                buf.resize(len.min(isize::MAX as usize), 0_u8);
            }
            stream.read_exact(&mut buf[..len]).await?;
            postcard::from_bytes(&buf[..len]).map_err(TheCurseIoError::Deserialization)
        }
    }
}

#[derive(Debug, Error)]
pub enum TheCurseIoError {
    #[error("Network error: {0}")]
    Network(#[from] smol::io::Error),
    #[error("Failed to serialize message: {0}")]
    Serialization(postcard::Error),
    #[error("Failed to deserialize message: {0}")]
    Deserialization(postcard::Error),
}
