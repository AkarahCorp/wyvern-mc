use std::{
    io::{Cursor, Error, ErrorKind, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
};

use zip::{ZipWriter, write::FileOptions};

use crate::TexturePack;

impl TexturePack {
    pub fn zip(&self) -> std::io::Result<Vec<u8>> {
        let buf = Vec::new();
        let buf_writer = Cursor::new(buf);
        let mut zip = ZipWriter::new(buf_writer);
        zip.set_flush_on_finish_file(true);

        zip.start_file::<&str, ()>("pack.mcmeta", FileOptions::default())
            .unwrap();

        zip.write_all(
            b"{
    \"pack\": {
        \"description\": {
            \"text\": \"server generated\"
        },
        \"pack_format\": 1
    }            
}",
        )?;

        self.zip_textures(&mut zip)?;

        zip.finish()
            .map(|x| x.into_inner())
            .map_err(|_| Error::from(ErrorKind::InvalidData))
    }

    pub fn zip_textures(&self, zip: &mut ZipWriter<Cursor<Vec<u8>>>) -> std::io::Result<()> {
        for texture in &self.textures {
            println!("{:#?}", texture.0);
            zip.start_file::<&str, ()>(
                &format!(
                    "assets/{}/textures/{}.png",
                    texture.0.namespace(),
                    texture.0.path()
                ),
                FileOptions::default(),
            )?;
            zip.write_all(texture.1)?;
        }
        Ok(())
    }

    pub fn host(&self) -> ! {
        let bytes = self.zip().unwrap();
        let server =
            TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 62000)).unwrap();
        loop {
            let mut client = server.accept().unwrap();
            client
                .0
                .write_all(
                    format!(
                        "{}\r\n{}\r\n{}{}\r\n\r\n",
                        "HTTP/1.1 200 OK",
                        "Content-Type: application/zip",
                        "Content-Length: ",
                        bytes.len()
                    )
                    .as_bytes(),
                )
                .unwrap();
            client.0.write_all(&bytes).unwrap();
            client.0.flush().unwrap();
        }
    }
}
