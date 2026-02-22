use anyhow::{Context, Result};
use std::fs::File;
use std::io::copy;
use std::path::Path;

/// Download genome from UCSC
pub fn download_genome(genome: &str, output_dir: &str) -> Result<String> {
    let url =
        get_genome_url(genome).ok_or_else(|| anyhow::anyhow!("Unknown genome: {}", genome))?;

    println!("Downloading {} from {}", genome, url);

    let output_path = Path::new(output_dir).join(format!("{}.fa", genome));

    // Download and decompress (if URL points to .gz file)
    if url.ends_with(".gz") {
        download_and_decompress(url, &output_path)?;
    } else {
        download_file(url, &output_path)?;
    }

    println!("Genome saved to: {}", output_path.display());
    Ok(output_path.to_string_lossy().to_string())
}

fn get_genome_url(genome: &str) -> Option<&'static str> {
    match genome.to_lowercase().as_str() {
        "hg19" => Some("https://hgdownload.cse.ucsc.edu/goldenPath/hg19/bigZips/hg19.fa.gz"),
        "hg38" => Some("https://hgdownload.cse.ucsc.edu/goldenPath/hg38/bigZips/hg38.fa.gz"),
        "mm10" => Some("https://hgdownload.cse.ucsc.edu/goldenPath/mm10/bigZips/mm10.fa.gz"),
        "mm39" => Some("https://hgdownload.cse.ucsc.edu/goldenPath/mm39/bigZips/mm39.fa.gz"),
        _ => None,
    }
}

fn download_and_decompress(url: &str, output_path: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    let response = reqwest::blocking::get(url).context("Failed to download genome")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download: HTTP {}", response.status());
    }

    let mut decoder = GzDecoder::new(response);
    let mut file = File::create(output_path)?;

    copy(&mut decoder, &mut file)?;

    Ok(())
}

fn download_file(url: &str, output_path: &Path) -> Result<()> {
    let response = reqwest::blocking::get(url).context("Failed to download file")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download: HTTP {}", response.status());
    }

    let mut file = File::create(output_path)?;
    let bytes = response.bytes()?;
    copy(&mut bytes.as_ref(), &mut file)?;

    Ok(())
}
