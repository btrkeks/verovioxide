use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub fn source_from_archive(
    url: &str,
    expected_sha256: &str,
    cached_archive: &Path,
    output: &Path,
    archive_root: &str,
) -> Result<PathBuf, String> {
    let bytes = if cached_archive.exists() {
        std::fs::read(cached_archive)
            .map_err(|error| format!("read pinned Verovio archive: {error}"))?
    } else {
        println!("cargo:warning=Downloading pinned Verovio source from {url}");
        ureq::get(url)
            .call()
            .map_err(|error| format!("download pinned Verovio source: {error}"))?
            .into_body()
            .read_to_vec()
            .map_err(|error| format!("read pinned Verovio response: {error}"))?
    };
    let actual_sha256 = format!("{:x}", Sha256::digest(&bytes));
    if actual_sha256 != expected_sha256 {
        return Err(format!(
            "pinned Verovio archive SHA256 mismatch: expected {expected_sha256}, found {actual_sha256}; remove {} to retry",
            cached_archive.display()
        ));
    }
    if !cached_archive.exists() {
        let parent = cached_archive
            .parent()
            .ok_or("Verovio archive cache needs a parent directory")?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("create Verovio archive cache: {error}"))?;
        let staging = cached_archive.with_extension(format!("download-{}", std::process::id()));
        std::fs::write(&staging, &bytes)
            .map_err(|error| format!("write pinned Verovio archive: {error}"))?;
        std::fs::rename(&staging, cached_archive)
            .map_err(|error| format!("cache pinned Verovio archive: {error}"))?;
    }
    if output.exists() {
        std::fs::remove_dir_all(output)
            .map_err(|error| format!("refresh extracted Verovio source: {error}"))?;
    }
    std::fs::create_dir_all(output)
        .map_err(|error| format!("create extracted Verovio source: {error}"))?;
    let decoder = flate2::read::GzDecoder::new(bytes.as_slice());
    tar::Archive::new(decoder)
        .unpack(output)
        .map_err(|error| format!("extract pinned Verovio archive: {error}"))?;
    let source = output.join(archive_root);
    if !source.join("src").is_dir() {
        return Err(format!(
            "pinned Verovio archive has no source at {}",
            source.display()
        ));
    }
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "verovio-archive-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn acquire(&self, url: &str, checksum: &str) -> Result<PathBuf, String> {
            source_from_archive(
                url,
                checksum,
                &self.0.join("cache/source.tar.gz"),
                &self.0.join("output"),
                "verovio-pinned",
            )
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn archive() -> Vec<u8> {
        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut archive = tar::Builder::new(encoder);
        let content = b"int probe = 1;\n";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive
            .append_data(
                &mut header,
                "verovio-pinned/src/probe.cpp",
                content.as_slice(),
            )
            .unwrap();
        archive.into_inner().unwrap().finish().unwrap()
    }

    fn serve(bytes: Vec<u8>, status: &str) -> (String, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/source.tar.gz", listener.local_addr().unwrap());
        let status = status.to_owned();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            stream.read(&mut request).unwrap();
            write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                bytes.len()
            )
            .unwrap();
            stream.write_all(&bytes).unwrap();
        });
        (url, server)
    }

    #[test]
    fn downloads_verified_source_then_reuses_archive_offline_and_refreshes_extraction() {
        let fixture = Fixture::new();
        let bytes = archive();
        let checksum = format!("{:x}", Sha256::digest(&bytes));
        let (url, server) = serve(bytes, "200 OK");
        let source = fixture.acquire(&url, &checksum).unwrap();
        server.join().unwrap();
        assert_eq!(
            std::fs::read_to_string(source.join("src/probe.cpp")).unwrap(),
            "int probe = 1;\n"
        );
        std::fs::write(source.join("src/probe.cpp"), "changed").unwrap();
        std::fs::write(source.join("src/extra.cpp"), "extra").unwrap();
        let refreshed = fixture
            .acquire("http://127.0.0.1:1/unavailable", &checksum)
            .unwrap();
        assert_eq!(source, refreshed);
        assert_eq!(
            std::fs::read_to_string(source.join("src/probe.cpp")).unwrap(),
            "int probe = 1;\n"
        );
        assert!(!source.join("src/extra.cpp").exists());
    }

    #[test]
    fn wrong_download_checksum_is_rejected_before_caching_or_extraction() {
        let fixture = Fixture::new();
        let (url, server) = serve(archive(), "200 OK");
        assert!(
            fixture
                .acquire(&url, "wrong")
                .unwrap_err()
                .contains("SHA256 mismatch")
        );
        server.join().unwrap();
        assert!(!fixture.0.join("cache").exists());
        assert!(!fixture.0.join("output").exists());
    }

    #[test]
    fn corrupt_cached_archive_is_rejected_without_network_or_extraction() {
        let fixture = Fixture::new();
        std::fs::create_dir(fixture.0.join("cache")).unwrap();
        std::fs::write(fixture.0.join("cache/source.tar.gz"), "corrupt").unwrap();
        assert!(
            fixture
                .acquire("http://127.0.0.1:1/unavailable", "expected")
                .unwrap_err()
                .contains("SHA256 mismatch")
        );
        assert!(!fixture.0.join("output").exists());
    }

    #[test]
    fn unavailable_pinned_archive_has_no_fallback_or_partial_cache() {
        let fixture = Fixture::new();
        let (url, server) = serve(Vec::new(), "404 Not Found");
        assert!(
            fixture
                .acquire(&url, "expected")
                .unwrap_err()
                .contains("download pinned Verovio source")
        );
        server.join().unwrap();
        assert!(!fixture.0.join("cache").exists());
        assert!(!fixture.0.join("output").exists());
    }
}
