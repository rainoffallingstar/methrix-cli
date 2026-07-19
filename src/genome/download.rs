#[cfg(feature = "download")]
use std::path::Path;

/// Download genome from UCSC.
/// Enabled via `cargo build --features download`.
#[cfg(feature = "download")]
pub fn download_genome(genome: &str, output_dir: &str) -> anyhow::Result<String> {
    let url =
        get_genome_url(genome).ok_or_else(|| anyhow::anyhow!("Unknown genome: {}", genome))?;

    println!("Downloading {} from {}", genome, url);

    let output_path = Path::new(output_dir).join(format!("{}.fa", genome));

    if url.ends_with(".gz") {
        download_and_decompress(url, &output_path)?;
    } else {
        download_file(url, &output_path)?;
    }

    println!("Genome saved to: {}", output_path.display());
    Ok(output_path.to_string_lossy().to_string())
}

#[cfg(feature = "download")]
fn get_genome_url(genome: &str) -> Option<&'static str> {
    match genome.to_lowercase().as_str() {
        "hg19" => Some("https://hgdownload.cse.ucsc.edu/goldenPath/hg19/bigZips/hg19.fa.gz"),
        "hg38" => Some("https://hgdownload.cse.ucsc.edu/goldenPath/hg38/bigZips/hg38.fa.gz"),
        "mm10" => Some("https://hgdownload.cse.ucsc.edu/goldenPath/mm10/bigZips/mm10.fa.gz"),
        "mm39" => Some("https://hgdownload.cse.ucsc.edu/goldenPath/mm39/bigZips/mm39.fa.gz"),
        _ => None,
    }
}

#[cfg(feature = "download")]
fn download_and_decompress(url: &str, output_path: &Path) -> anyhow::Result<()> {
    use anyhow::Context;
    use flate2::read::GzDecoder;
    use std::fs::File;
    use std::io::copy;

    let response = reqwest::blocking::get(url).context("Failed to download genome")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download: HTTP {}", response.status());
    }

    let mut decoder = GzDecoder::new(response);
    let mut file = File::create(output_path)?;

    copy(&mut decoder, &mut file)?;

    Ok(())
}

#[cfg(feature = "download")]
fn download_file(url: &str, output_path: &Path) -> anyhow::Result<()> {
    use anyhow::Context;
    use std::fs::File;
    use std::io::copy;

    let response = reqwest::blocking::get(url).context("Failed to download file")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download: HTTP {}", response.status());
    }

    let mut file = File::create(output_path)?;
    let bytes = response.bytes()?;
    copy(&mut bytes.as_ref(), &mut file)?;

    Ok(())
}

#[cfg(all(test, feature = "download"))]
mod tests {
    use super::get_genome_url;

    #[test]
    fn resolves_supported_ucsc_genomes_case_insensitively() {
        assert_eq!(
            get_genome_url("HG38"),
            Some("https://hgdownload.cse.ucsc.edu/goldenPath/hg38/bigZips/hg38.fa.gz")
        );
        assert!(get_genome_url("unsupported").is_none());
    }
}
