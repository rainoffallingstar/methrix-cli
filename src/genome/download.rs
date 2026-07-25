#[cfg(feature = "download")]
use anyhow::{bail, Context, Result};
#[cfg(feature = "download")]
use md5::{Digest, Md5};
#[cfg(feature = "download")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "download")]
use std::fs::{self, File};
#[cfg(feature = "download")]
use std::io::{copy, sink, Read, Write};
#[cfg(feature = "download")]
use std::path::{Path, PathBuf};
#[cfg(feature = "download")]
use std::time::Duration;

#[cfg(feature = "download")]
use crate::atomic_output::AtomicOutputSet;

#[cfg(feature = "download")]
const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
#[cfg(feature = "download")]
const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(30 * 60);
#[cfg(feature = "download")]
const MAX_DOWNLOAD_BYTES: u64 = 8 * 1024 * 1024 * 1024;
#[cfg(feature = "download")]
const MAX_DECOMPRESSED_BYTES: u64 = 16 * 1024 * 1024 * 1024;
#[cfg(feature = "download")]
const PROVENANCE_SCHEMA_NAME: &str = "methx.genome-download";
#[cfg(feature = "download")]
const PROVENANCE_SCHEMA_VERSION: &str = "1.0.0";

#[cfg(feature = "download")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GenomeRelease {
    name: &'static str,
    url: &'static str,
    source_md5: &'static str,
}

// Fixed digests are from UCSC's official HTTPS bigZips/md5sum.txt manifests,
// verified 2026-07-24. A release update must change the URL and digest together.
#[cfg(feature = "download")]
const GENOME_RELEASES: [GenomeRelease; 4] = [
    GenomeRelease {
        name: "hg19",
        url: "https://hgdownload.soe.ucsc.edu/goldenPath/hg19/bigZips/hg19.fa.gz",
        source_md5: "806c02398f5ac5da8ffd6da2d1d5d1a9",
    },
    GenomeRelease {
        name: "hg38",
        url: "https://hgdownload.soe.ucsc.edu/goldenPath/hg38/bigZips/hg38.fa.gz",
        source_md5: "1c9dcaddfa41027f17cd8f7a82c7293b",
    },
    GenomeRelease {
        name: "mm10",
        url: "https://hgdownload.soe.ucsc.edu/goldenPath/mm10/bigZips/mm10.fa.gz",
        source_md5: "db005b65828db31735f384e4c5787be5",
    },
    GenomeRelease {
        name: "mm39",
        url: "https://hgdownload.soe.ucsc.edu/goldenPath/mm39/bigZips/mm39.fa.gz",
        source_md5: "41ace1b7157b98b393746aef5d1287a8",
    },
];

#[cfg(feature = "download")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenomeDownloadProvenance {
    pub schema_name: String,
    pub schema_version: String,
    pub genome_release: String,
    pub source_url: String,
    pub source_md5: String,
    pub source_bytes: u64,
    pub fasta_md5: String,
    pub fasta_bytes: u64,
}

#[cfg(feature = "download")]
#[derive(Debug)]
struct DownloadMeasurements {
    source_md5: String,
    source_bytes: u64,
    fasta_md5: String,
    fasta_bytes: u64,
}

/// Download a pinned UCSC genome, verify the compressed source digest, and
/// atomically publish both the FASTA and its locally verifiable provenance.
/// Enabled via `cargo build --features download`.
#[cfg(feature = "download")]
pub fn download_genome(genome: &str, output_dir: &str) -> Result<String> {
    let release =
        get_genome_release(genome).ok_or_else(|| anyhow::anyhow!("Unknown genome: {}", genome))?;
    let output_directory = Path::new(output_dir);
    fs::create_dir_all(output_directory).with_context(|| {
        format!(
            "Failed to create genome output directory {}",
            output_directory.display()
        )
    })?;
    let output_path = output_directory.join(format!("{}.fa", release.name));
    let provenance_path = provenance_path_for(&output_path);

    if output_path.is_file() && provenance_path.is_file() {
        match validate_cached_genome(&output_path, &provenance_path, release) {
            Ok(()) => {
                println!("Using verified cached genome: {}", output_path.display());
                return Ok(output_path.to_string_lossy().into_owned());
            }
            Err(error) => eprintln!(
                "warning: cached genome failed validation and will be replaced: {:#}",
                error
            ),
        }
    }

    let client = reqwest::blocking::Client::builder()
        .connect_timeout(HTTP_CONNECT_TIMEOUT)
        .timeout(HTTP_REQUEST_TIMEOUT)
        .build()
        .context("Failed to configure HTTP client")?;

    println!("Downloading {} from {}", release.name, release.url);
    let mut output_set = AtomicOutputSet::new(output_directory)?;
    let mut measurements = None;
    output_set.stage(&output_path, |temporary_path| {
        measurements = Some(download_and_decompress(&client, release, temporary_path)?);
        Ok(())
    })?;
    let measurements = measurements.context("Genome download produced no measurements")?;
    let provenance = GenomeDownloadProvenance {
        schema_name: PROVENANCE_SCHEMA_NAME.to_string(),
        schema_version: PROVENANCE_SCHEMA_VERSION.to_string(),
        genome_release: release.name.to_string(),
        source_url: release.url.to_string(),
        source_md5: measurements.source_md5,
        source_bytes: measurements.source_bytes,
        fasta_md5: measurements.fasta_md5,
        fasta_bytes: measurements.fasta_bytes,
    };
    output_set.stage(&provenance_path, |temporary_path| {
        let encoded = ron::ser::to_string_pretty(&provenance, Default::default())
            .context("Failed to serialize genome provenance")?;
        fs::write(temporary_path, encoded).with_context(|| {
            format!(
                "Failed to stage genome provenance {}",
                provenance_path.display()
            )
        })?;
        Ok(())
    })?;
    output_set.publish()?;

    println!("Genome saved to: {}", output_path.display());
    println!("Provenance saved to: {}", provenance_path.display());
    Ok(output_path.to_string_lossy().into_owned())
}

#[cfg(feature = "download")]
fn get_genome_release(genome: &str) -> Option<GenomeRelease> {
    GENOME_RELEASES
        .iter()
        .copied()
        .find(|release| release.name.eq_ignore_ascii_case(genome))
}

#[cfg(feature = "download")]
fn provenance_path_for(fasta_path: &Path) -> PathBuf {
    let file_name = fasta_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("genome.fa");
    fasta_path.with_file_name(format!("{}.provenance.ron", file_name))
}

#[cfg(feature = "download")]
fn send_download_request(
    client: &reqwest::blocking::Client,
    release: GenomeRelease,
) -> Result<reqwest::blocking::Response> {
    let response = client
        .get(release.url)
        .send()
        .with_context(|| format!("Failed to download {}", release.url))?
        .error_for_status()
        .with_context(|| format!("Genome download returned an error for {}", release.url))?;

    if let Some(content_length) = response.content_length() {
        if content_length > MAX_DOWNLOAD_BYTES {
            bail!(
                "Genome download declares {} bytes, exceeding the {} byte limit",
                content_length,
                MAX_DOWNLOAD_BYTES
            );
        }
    }

    Ok(response)
}

#[cfg(feature = "download")]
fn download_and_decompress(
    client: &reqwest::blocking::Client,
    release: GenomeRelease,
    output_path: &Path,
) -> Result<DownloadMeasurements> {
    use flate2::read::GzDecoder;

    let response = send_download_request(client, release)?;
    let source = DigestingReader::new(response.take(MAX_DOWNLOAD_BYTES + 1));
    let mut decoder = GzDecoder::new(source);
    let output_file = File::create(output_path)
        .with_context(|| format!("Failed to create {}", output_path.display()))?;
    let mut output = DigestingWriter::new(output_file);
    copy_with_limit(
        &mut decoder,
        &mut output,
        MAX_DECOMPRESSED_BYTES,
        "decompressed genome",
    )?;

    // Consume any trailing compressed bytes so the source digest covers the
    // complete HTTP payload rather than only the first gzip member.
    let mut source = decoder.into_inner();
    copy(&mut source, &mut sink()).context("Failed to finish hashing genome source")?;
    let (source_md5, source_bytes) = source.finish();
    if source_bytes > MAX_DOWNLOAD_BYTES {
        bail!(
            "Genome download exceeded the configured {} byte limit",
            MAX_DOWNLOAD_BYTES
        );
    }
    if source_md5 != release.source_md5 {
        bail!(
            "Checksum mismatch for {}: expected {}, received {}",
            release.url,
            release.source_md5,
            source_md5
        );
    }
    let (fasta_md5, fasta_bytes) = output.finish()?;

    Ok(DownloadMeasurements {
        source_md5,
        source_bytes,
        fasta_md5,
        fasta_bytes,
    })
}

#[cfg(feature = "download")]
fn validate_cached_genome(
    fasta_path: &Path,
    provenance_path: &Path,
    release: GenomeRelease,
) -> Result<()> {
    let provenance_text = fs::read_to_string(provenance_path).with_context(|| {
        format!(
            "Failed to read genome provenance {}",
            provenance_path.display()
        )
    })?;
    let provenance: GenomeDownloadProvenance =
        ron::from_str(&provenance_text).context("Failed to parse genome provenance")?;
    if provenance.schema_name != PROVENANCE_SCHEMA_NAME
        || provenance.schema_version != PROVENANCE_SCHEMA_VERSION
        || provenance.genome_release != release.name
        || provenance.source_url != release.url
        || provenance.source_md5 != release.source_md5
    {
        bail!("Genome provenance does not match the pinned release manifest");
    }
    if provenance.fasta_bytes > MAX_DECOMPRESSED_BYTES {
        bail!("Cached genome exceeds the configured decompressed size limit");
    }
    let file_size = fs::metadata(fasta_path)
        .with_context(|| format!("Failed to stat cached genome {}", fasta_path.display()))?
        .len();
    if file_size != provenance.fasta_bytes {
        bail!(
            "Cached genome size mismatch: expected {}, found {}",
            provenance.fasta_bytes,
            file_size
        );
    }
    let (fasta_md5, fasta_bytes) = digest_file(fasta_path, MAX_DECOMPRESSED_BYTES)?;
    if fasta_bytes != provenance.fasta_bytes || fasta_md5 != provenance.fasta_md5 {
        bail!(
            "Cached genome digest mismatch: expected {}, found {}",
            provenance.fasta_md5,
            fasta_md5
        );
    }
    Ok(())
}

#[cfg(feature = "download")]
fn digest_file(path: &Path, maximum_bytes: u64) -> Result<(String, u64)> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open cached genome {}", path.display()))?;
    let mut reader = DigestingReader::new(file.take(maximum_bytes + 1));
    copy(&mut reader, &mut sink())?;
    let result = reader.finish();
    if result.1 > maximum_bytes {
        bail!("Cached genome exceeded the configured byte limit");
    }
    Ok(result)
}

#[cfg(feature = "download")]
fn copy_with_limit<R: Read, W: Write>(
    reader: R,
    writer: &mut W,
    maximum_bytes: u64,
    content_name: &str,
) -> Result<u64> {
    let mut limited_reader = reader.take(maximum_bytes + 1);
    let copied_bytes = copy(&mut limited_reader, writer)?;
    if copied_bytes > maximum_bytes {
        bail!(
            "{} exceeded the configured {} byte limit",
            content_name,
            maximum_bytes
        );
    }
    Ok(copied_bytes)
}

#[cfg(feature = "download")]
fn finalize_md5(hasher: Md5) -> String {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
    let digest = hasher.finalize();
    let digest_bytes: &[u8] = digest.as_ref();
    let mut encoded = String::with_capacity(digest_bytes.len() * 2);
    for &byte in digest_bytes {
        encoded.push(HEX_DIGITS[(byte >> 4) as usize] as char);
        encoded.push(HEX_DIGITS[(byte & 0x0f) as usize] as char);
    }
    encoded
}

#[cfg(feature = "download")]
struct DigestingReader<R> {
    inner: R,
    hasher: Md5,
    bytes: u64,
}

#[cfg(feature = "download")]
impl<R> DigestingReader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            hasher: Md5::new(),
            bytes: 0,
        }
    }

    fn finish(self) -> (String, u64) {
        (finalize_md5(self.hasher), self.bytes)
    }
}

#[cfg(feature = "download")]
impl<R: Read> Read for DigestingReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.inner.read(buffer)?;
        self.hasher.update(&buffer[..bytes_read]);
        self.bytes = self.bytes.saturating_add(bytes_read as u64);
        Ok(bytes_read)
    }
}

#[cfg(feature = "download")]
struct DigestingWriter<W> {
    inner: W,
    hasher: Md5,
    bytes: u64,
}

#[cfg(feature = "download")]
impl<W> DigestingWriter<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            hasher: Md5::new(),
            bytes: 0,
        }
    }
}

#[cfg(feature = "download")]
impl<W: Write> DigestingWriter<W> {
    fn finish(mut self) -> std::io::Result<(String, u64)> {
        self.inner.flush()?;
        Ok((finalize_md5(self.hasher), self.bytes))
    }
}

#[cfg(feature = "download")]
impl<W: Write> Write for DigestingWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let bytes_written = self.inner.write(buffer)?;
        self.hasher.update(&buffer[..bytes_written]);
        self.bytes = self.bytes.saturating_add(bytes_written as u64);
        Ok(bytes_written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(all(test, feature = "download"))]
mod tests {
    use super::{
        copy_with_limit, digest_file, get_genome_release, provenance_path_for,
        validate_cached_genome, GenomeDownloadProvenance, PROVENANCE_SCHEMA_NAME,
        PROVENANCE_SCHEMA_VERSION,
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn resolves_supported_ucsc_genomes_with_pinned_digests() {
        let release = get_genome_release("HG38").unwrap();
        assert_eq!(release.name, "hg38");
        assert_eq!(release.source_md5, "1c9dcaddfa41027f17cd8f7a82c7293b");
        assert!(release.url.ends_with("/hg38.fa.gz"));
        assert!(get_genome_release("unsupported").is_none());
    }

    #[test]
    fn rejects_streams_above_download_limit() {
        let source = [1u8, 2, 3, 4];
        let mut destination = Vec::new();
        let error =
            copy_with_limit(source.as_slice(), &mut destination, 3, "test payload").unwrap_err();
        assert!(error.to_string().contains("exceeded"));
    }

    #[test]
    fn validates_cache_and_rejects_modified_fasta() {
        let directory = tempdir().unwrap();
        let fasta_path = directory.path().join("hg38.fa");
        let provenance_path = provenance_path_for(&fasta_path);
        fs::write(&fasta_path, b"abc").unwrap();
        let (fasta_md5, fasta_bytes) = digest_file(&fasta_path, 100).unwrap();
        let release = get_genome_release("hg38").unwrap();
        let provenance = GenomeDownloadProvenance {
            schema_name: PROVENANCE_SCHEMA_NAME.to_string(),
            schema_version: PROVENANCE_SCHEMA_VERSION.to_string(),
            genome_release: release.name.to_string(),
            source_url: release.url.to_string(),
            source_md5: release.source_md5.to_string(),
            source_bytes: 3,
            fasta_md5,
            fasta_bytes,
        };
        fs::write(&provenance_path, ron::ser::to_string(&provenance).unwrap()).unwrap();

        validate_cached_genome(&fasta_path, &provenance_path, release).unwrap();
        fs::write(&fasta_path, b"abd").unwrap();
        let error = validate_cached_genome(&fasta_path, &provenance_path, release).unwrap_err();
        assert!(error.to_string().contains("digest mismatch"));
    }
}
