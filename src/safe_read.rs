use anyhow::bail;
use esp_idf_svc::io::Read;

pub trait SafeRead {
    fn try_next(&mut self) -> anyhow::Result<u8> {
        Ok(self.try_read::<1>()?[0])
    }

    fn try_read<const N: usize>(&mut self) -> anyhow::Result<[u8; N]>;
    fn try_read_dyn(&mut self, n: usize) -> anyhow::Result<Vec<u8>>;
}

impl<R: Read> SafeRead for R {
    fn try_read<const N: usize>(&mut self) -> anyhow::Result<[u8; N]> {
        let mut buf = [0_u8; N];
        let size_read = match self.read(&mut buf) {
            Ok(s) => s,
            Err(e) => bail!("Read Error: {e:?}"),
        };

        if size_read != N {
            bail!("Size mismatch between read size ({size_read}) and expected {N}");
        }

        Ok(buf)
    }

    fn try_read_dyn(&mut self, n: usize) -> anyhow::Result<Vec<u8>> {
        let mut buf = vec![0; n];
        let size_read = match self.read(&mut buf) {
            Ok(s) => s,
            Err(e) => bail!("Read Error: {e:?}"),
        };

        if size_read != n {
            bail!("Size mismatch between read size ({size_read}) and expected {n}");
        }

        Ok(buf)
    }
}
