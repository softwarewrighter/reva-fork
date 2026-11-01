//! `(save)`: the system as an image file.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::vfs::Access;
use reva_core_boot::image::Images;
use reva_words_sys::fs::{ret, with_fs};
use reva_words_sys::paths::Paths;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Saving an image.
    pub trait Save: Paths {
        /// `(save)` ( a n -- ): write an image of the system to the file named
        /// by `a n` (see `image.rs`).  The original wrote a native executable;
        /// an image is its portable form, started with `reva --image` or by a
        /// page (`start(..., { image })`).  `ioerr` as for the file words.
        fn word_save(&mut self, host: &mut dyn Host) -> R {
            let n = self.upop()?;
            let a = self.upop()?;
            let path = self.path_at(a, n)?;
            let image = self.save_image();
            let r = ret(with_fs(host, |fs| {
                let fd = fs.open(&path, Access::Create)?;
                let w = fs.write(fd, &image);
                fs.close(fd)?;
                w.map(|_| 0)
            }));
            self.set_ior(r)
        }
    }
}
