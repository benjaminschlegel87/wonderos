use core::future::Future;
use core::task::Poll;
use embedded_hal::spi::FullDuplex;
pub async fn async_read<T: FullDuplex<u8>>(spi: &mut T) -> Result<u8, ()> {
    AsynSpiRead { spi }.await
}
pub async fn async_write<T: FullDuplex<u8>>(spi: &mut T, payload: u8) -> Result<(), ()> {
    AsynSpiWrite { spi, payload }.await
}
pub async fn async_transfer<'a, T: FullDuplex<u8>>(
    spi: &mut T,
    transfer_buffer: &'a mut [u8],
) -> Result<&'a [u8], ()> {
    for byte in transfer_buffer.iter_mut() {
        async_write(spi, *byte).await?;
        *byte = async_read(spi).await?;
    }
    Ok(transfer_buffer)
}
struct AsynSpiRead<'a, T: FullDuplex<u8>> {
    spi: &'a mut T,
}

impl<'a, T: FullDuplex<u8>> Future for AsynSpiRead<'a, T> {
    type Output = Result<u8, ()>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let r = self.spi.read();
        match r {
            Ok(res) => Poll::Ready(Ok(res)),
            Err(nb::Error::Other(_)) => Poll::Ready(Err(())),
            Err(nb::Error::WouldBlock) => {
                return Poll::Pending;
            }
        }
    }
}

struct AsynSpiWrite<'a, T: FullDuplex<u8>> {
    spi: &'a mut T,
    payload: u8,
}
impl<'a, T: FullDuplex<u8>> Future for AsynSpiWrite<'a, T> {
    type Output = Result<(), ()>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let p = self.payload;
        let r = self.spi.send(p);
        match r {
            Ok(_) => Poll::Ready(Ok(())),
            Err(nb::Error::Other(_)) => Poll::Ready(Err(())),
            Err(nb::Error::WouldBlock) => Poll::Pending,
        }
    }
}
