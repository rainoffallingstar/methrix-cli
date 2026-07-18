use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use ndarray::Array2;
use rust_xlsxwriter::Workbook;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::genome::cpg::CpGSite;

const PROMOTER_WINDOW: u32 = 3_000;
const DOWNSTREAM_WINDOW: u32 = 3_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GenomicAnnotation {
    Promoter,
    FivePrimeUtr,
    ThreePrimeUtr,
    Exon,
    Intron,
    Downstream,
    Intergenic,
}

impl GenomicAnnotation {
    fn as_str(self) -> &'static str {
        match self {
            GenomicAnnotation::Promoter => "Promoter",
            GenomicAnnotation::FivePrimeUtr => "5' UTR",
            GenomicAnnotation::ThreePrimeUtr => "3' UTR",
            GenomicAnnotation::Exon => "Exon",
            GenomicAnnotation::Intron => "Intron",
            GenomicAnnotation::Downstream => "Downstream",
            GenomicAnnotation::Intergenic => "Intergenic",
        }
    }

    fn priority(self) -> u8 {
        match self {
            GenomicAnnotation::Promoter => 1,
            GenomicAnnotation::FivePrimeUtr => 2,
            GenomicAnnotation::ThreePrimeUtr => 3,
            GenomicAnnotation::Exon => 4,
            GenomicAnnotation::Intron => 5,
            GenomicAnnotation::Downstream => 6,
            GenomicAnnotation::Intergenic => 7,
        }
    }
}

#[derive(Debug, Clone)]
struct Interval {
    start: u32,
    end: u32,
}

impl Interval {
    fn contains(&self, pos: u32) -> bool {
        self.start <= pos && pos < self.end
    }
}

#[derive(Debug, Clone)]
struct RankedInterval {
    interval: Interval,
    rank_label: String,
}

#[derive(Debug, Clone)]
struct TranscriptModel {
    strand: char,
    transcript_id: String,
    gene_id: String,
    gene_symbol: String,
    tx_start: u32,
    tx_end: u32,
    tss: u32,
    promoter: Interval,
    downstream: Interval,
    exons: Vec<RankedInterval>,
    introns: Vec<RankedInterval>,
    utr5: Vec<Interval>,
    utr3: Vec<Interval>,
}

#[derive(Debug, Clone)]
struct TranscriptBuilder {
    chr: String,
    strand: char,
    transcript_id: String,
    gene_id: String,
    gene_symbol: String,
    tx_start: Option<u32>,
    tx_end: Option<u32>,
    exons: Vec<Interval>,
    utr5: Vec<Interval>,
    utr3: Vec<Interval>,
}

#[derive(Debug, Clone)]
struct GeneAnnotations {
    transcripts_by_chr: HashMap<String, Vec<TranscriptModel>>,
}

#[derive(Debug, Clone)]
struct AnnotationResources {
    genes: GeneAnnotations,
}

#[derive(Debug, Clone)]
pub struct AnnotationRecord {
    pub chr: String,
    pub start_1based: u32,
    pub end_1based: u32,
    pub strand: char,
    pub annotation: String,
    pub gene_id: String,
    pub gene_symbol: String,
    pub transcript_id: String,
    pub distance_to_tss: i32,
    pub exon_intron_rank: String,
}

#[derive(Debug, Clone)]
pub struct AnnotationResult {
    pub records: Vec<AnnotationRecord>,
    pub sample_summary: Vec<SampleAnnotationRow>,
}

#[derive(Debug, Clone)]
pub struct SampleAnnotationRow {
    pub sample_name: String,
    pub covered_cpgs: usize,
    pub annotation_counts: BTreeMap<String, usize>,
}

impl AnnotationResult {
    pub fn write_excel_report(&self, output_path: &str) -> Result<()> {
        let mut workbook = Workbook::new();

        let _sheet_by_sample = workbook.add_worksheet();
        let sheet_by_sample = workbook
            .worksheet_from_index(0)
            .context("Failed to get by-sample worksheet")?;
        sheet_by_sample.set_name("ChIPseeker_By_Sample")?;

        let mut annotations: Vec<String> =
            self.records.iter().map(|r| r.annotation.clone()).collect();
        annotations.sort();
        annotations.dedup();

        sheet_by_sample.write_string(0, 0, "sample")?;
        sheet_by_sample.write_string(0, 1, "covered_cpgs")?;
        let mut col = 2u16;
        for ann in &annotations {
            sheet_by_sample.write_string(0, col, format!("{}_count", ann))?;
            sheet_by_sample.write_string(0, col + 1, format!("{}_percent", ann))?;
            col += 2;
        }

        for (idx, sample_row) in self.sample_summary.iter().enumerate() {
            let row = (idx + 1) as u32;
            sheet_by_sample.write_string(row, 0, &sample_row.sample_name)?;
            sheet_by_sample.write_number(row, 1, sample_row.covered_cpgs as f64)?;

            let mut col = 2u16;
            for ann in &annotations {
                let count = *sample_row.annotation_counts.get(ann).unwrap_or(&0usize);
                let pct = if sample_row.covered_cpgs > 0 {
                    count as f64 * 100.0 / sample_row.covered_cpgs as f64
                } else {
                    0.0
                };
                sheet_by_sample.write_number(row, col, count as f64)?;
                sheet_by_sample.write_number(row, col + 1, pct)?;
                col += 2;
            }
        }

        let _sheet_details = workbook.add_worksheet();
        let sheet_details = workbook
            .worksheet_from_index(1)
            .context("Failed to get details worksheet")?;
        sheet_details.set_name("CpG_Details")?;

        let headers = [
            "chr",
            "start_1based",
            "end_1based",
            "strand",
            "annotation",
            "gene_id",
            "gene_symbol",
            "transcript_id",
            "distance_to_tss",
            "exon_intron_rank",
        ];
        for (col, header) in headers.iter().enumerate() {
            sheet_details.write_string(0, col as u16, *header)?;
        }

        for (idx, rec) in self.records.iter().enumerate() {
            let row = (idx + 1) as u32;
            sheet_details.write_string(row, 0, &rec.chr)?;
            sheet_details.write_number(row, 1, rec.start_1based as f64)?;
            sheet_details.write_number(row, 2, rec.end_1based as f64)?;
            sheet_details.write_string(row, 3, rec.strand.to_string())?;
            sheet_details.write_string(row, 4, &rec.annotation)?;
            sheet_details.write_string(row, 5, &rec.gene_id)?;
            sheet_details.write_string(row, 6, &rec.gene_symbol)?;
            sheet_details.write_string(row, 7, &rec.transcript_id)?;
            sheet_details.write_number(row, 8, rec.distance_to_tss as f64)?;
            sheet_details.write_string(row, 9, &rec.exon_intron_rank)?;
        }

        workbook.save(output_path)?;
        Ok(())
    }
}

pub fn annotate_cpgs(
    cpg_sites: &[CpGSite],
    cov_matrix: &Array2<u32>,
    sample_names: &[String],
    genome: &str,
    annotation_dir: Option<&str>,
) -> Result<AnnotationResult> {
    let resources = load_annotation_resources(genome, annotation_dir)?;

    let mut records = Vec::with_capacity(cpg_sites.len());
    for cpg in cpg_sites {
        let chip = annotate_genomic_feature(&resources.genes, &cpg.chr, cpg.start);

        records.push(AnnotationRecord {
            chr: cpg.chr.clone(),
            start_1based: cpg.start + 1,
            end_1based: cpg.end,
            strand: cpg.strand,
            annotation: chip.annotation.as_str().to_string(),
            gene_id: chip.gene_id,
            gene_symbol: chip.gene_symbol,
            transcript_id: chip.transcript_id,
            distance_to_tss: chip.distance_to_tss,
            exon_intron_rank: chip.exon_intron_rank,
        });
    }

    let sample_summary = calculate_sample_annotation_summary(&records, cov_matrix, sample_names)?;

    Ok(AnnotationResult {
        records,
        sample_summary,
    })
}

fn calculate_sample_annotation_summary(
    records: &[AnnotationRecord],
    cov_matrix: &Array2<u32>,
    sample_names: &[String],
) -> Result<Vec<SampleAnnotationRow>> {
    let (n_rows, n_samples) = cov_matrix.dim();
    if n_rows != records.len() || n_samples != sample_names.len() {
        anyhow::bail!(
            "Annotation matrix dimensions {:?} do not match {} records and {} samples",
            cov_matrix.dim(),
            records.len(),
            sample_names.len()
        );
    }

    let mut rows = Vec::new();
    for (sample_idx, sample_name) in sample_names.iter().enumerate() {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut covered_cpgs = 0usize;

        for row_idx in 0..n_rows {
            if cov_matrix[(row_idx, sample_idx)] > 0 {
                covered_cpgs += 1;
                let ann = records[row_idx].annotation.clone();
                *counts.entry(ann).or_insert(0) += 1;
            }
        }

        rows.push(SampleAnnotationRow {
            sample_name: sample_name.clone(),
            covered_cpgs,
            annotation_counts: counts,
        });
    }

    Ok(rows)
}

fn load_annotation_resources(
    genome: &str,
    annotation_dir: Option<&str>,
) -> Result<AnnotationResources> {
    let genome_key = normalize_genome_name(genome);
    let inferred_dir = Path::new(genome)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .and_then(|parent| parent.to_str())
        .map(str::to_string);
    let dir = annotation_dir
        .map(str::to_string)
        .or(inferred_dir)
        .context(
            "Annotation requires --annotation-dir unless --genome includes a parent directory",
        )?;
    let dir_path = Path::new(&dir);
    let gtf_path = find_gtf_path(dir_path, &genome_key).with_context(|| {
        format!(
            "Expected {}.gtf/.gtf.gz or a detectable *.gtf(.gz) in {}",
            genome_key,
            dir_path.display()
        )
    })?;

    let genes = load_genes_from_gtf(&gtf_path)
        .with_context(|| format!("Failed to load gene annotation from {}", gtf_path.display()))?;

    Ok(AnnotationResources { genes })
}

fn find_gtf_path(dir: &Path, genome_key: &str) -> Result<PathBuf> {
    let plain = dir.join(format!("{}.gtf", genome_key));
    if plain.exists() {
        return Ok(plain);
    }

    let gz = dir.join(format!("{}.gtf.gz", genome_key));
    if gz.exists() {
        return Ok(gz);
    }

    // Fallback: if there's exactly one *.gtf or *.gtf.gz in dir, use it.
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            let lower = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            lower.ends_with(".gtf") || lower.ends_with(".gtf.gz")
        })
        .collect();
    candidates.sort();

    match candidates.len() {
        1 => Ok(candidates.remove(0)),
        0 => anyhow::bail!("GTF file not found"),
        _ => anyhow::bail!(
            "Multiple GTF candidates found; please specify --annotation-dir containing a unique target: {}",
            candidates
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn normalize_genome_name(genome: &str) -> String {
    let raw = Path::new(genome)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(genome)
        .to_ascii_lowercase();

    ["hg19", "hg38", "mm10", "mm39"]
        .iter()
        .find(|k| raw.contains(**k))
        .unwrap_or(&raw.as_str())
        .to_string()
}

fn load_genes_from_gtf(path: &Path) -> Result<GeneAnnotations> {
    let reader: Box<dyn BufRead> = if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.eq_ignore_ascii_case("gz"))
        .unwrap_or(false)
    {
        let file = File::open(path)?;
        Box::new(BufReader::new(GzDecoder::new(file)))
    } else {
        Box::new(BufReader::new(File::open(path)?))
    };

    let mut builders: HashMap<String, TranscriptBuilder> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 9 {
            continue;
        }

        let chr = fields[0].to_string();
        let feature = fields[2].to_ascii_lowercase();
        let start_1: u32 = match fields[3].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let end_1: u32 = match fields[4].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        if end_1 < start_1 {
            continue;
        }

        let start = start_1.saturating_sub(1);
        let end = end_1;
        let strand = fields[6].chars().next().unwrap_or('+');
        let attrs = parse_gtf_attributes(fields[8]);

        let transcript_id = attrs
            .get("transcript_id")
            .cloned()
            .or_else(|| attrs.get("gene_id").cloned())
            .unwrap_or_default();
        let gene_id = attrs.get("gene_id").cloned().unwrap_or_default();
        let gene_symbol = attrs
            .get("gene_name")
            .cloned()
            .unwrap_or_else(|| gene_id.clone());

        if transcript_id.is_empty() || gene_id.is_empty() {
            continue;
        }

        let builder = builders
            .entry(transcript_id.clone())
            .or_insert_with(|| TranscriptBuilder {
                chr: chr.clone(),
                strand,
                transcript_id: transcript_id.clone(),
                gene_id: gene_id.clone(),
                gene_symbol: gene_symbol.clone(),
                tx_start: None,
                tx_end: None,
                exons: Vec::new(),
                utr5: Vec::new(),
                utr3: Vec::new(),
            });

        builder.chr = chr;
        builder.strand = strand;
        if builder.gene_id.is_empty() {
            builder.gene_id = gene_id;
        }
        if builder.gene_symbol.is_empty() {
            builder.gene_symbol = gene_symbol;
        }

        match feature.as_str() {
            "transcript" | "mrna" => {
                builder.tx_start = Some(start);
                builder.tx_end = Some(end);
            }
            "exon" => builder.exons.push(Interval { start, end }),
            "five_prime_utr" | "5utr" | "5'utr" => builder.utr5.push(Interval { start, end }),
            "three_prime_utr" | "3utr" | "3'utr" => builder.utr3.push(Interval { start, end }),
            _ => {}
        }
    }

    let mut transcripts_by_chr: HashMap<String, Vec<TranscriptModel>> = HashMap::new();

    for (_, mut builder) in builders {
        if builder.exons.is_empty() && (builder.tx_start.is_none() || builder.tx_end.is_none()) {
            continue;
        }

        builder.exons.sort_by_key(|e| e.start);

        let tx_start = builder
            .tx_start
            .unwrap_or_else(|| builder.exons.first().map(|e| e.start).unwrap_or(0));
        let tx_end = builder
            .tx_end
            .unwrap_or_else(|| builder.exons.last().map(|e| e.end).unwrap_or(tx_start + 1));

        if tx_end <= tx_start {
            continue;
        }

        let tss = if builder.strand == '-' {
            tx_end.saturating_sub(1)
        } else {
            tx_start
        };

        let promoter = Interval {
            start: tss.saturating_sub(PROMOTER_WINDOW),
            end: tss.saturating_add(PROMOTER_WINDOW).saturating_add(1),
        };

        let downstream = if builder.strand == '-' {
            Interval {
                start: tx_start.saturating_sub(DOWNSTREAM_WINDOW),
                end: tx_start,
            }
        } else {
            Interval {
                start: tx_end,
                end: tx_end.saturating_add(DOWNSTREAM_WINDOW),
            }
        };

        let exons = build_ranked_intervals(&builder.exons, builder.strand, "Exon");
        let introns = build_introns(&builder.exons, builder.strand);

        let model = TranscriptModel {
            strand: builder.strand,
            transcript_id: builder.transcript_id.clone(),
            gene_id: builder.gene_id.clone(),
            gene_symbol: builder.gene_symbol.clone(),
            tx_start,
            tx_end,
            tss,
            promoter,
            downstream,
            exons,
            introns,
            utr5: builder.utr5,
            utr3: builder.utr3,
        };

        transcripts_by_chr
            .entry(builder.chr)
            .or_default()
            .push(model);
    }

    for txs in transcripts_by_chr.values_mut() {
        txs.sort_by(|a, b| {
            a.tx_start
                .cmp(&b.tx_start)
                .then_with(|| a.tx_end.cmp(&b.tx_end))
        });
    }

    Ok(GeneAnnotations { transcripts_by_chr })
}

fn build_ranked_intervals(
    intervals: &[Interval],
    strand: char,
    label: &str,
) -> Vec<RankedInterval> {
    let n = intervals.len();
    intervals
        .iter()
        .enumerate()
        .map(|(i, interval)| {
            let rank = if strand == '-' { n - i } else { i + 1 };
            RankedInterval {
                interval: interval.clone(),
                rank_label: format!("{} {} of {}", label, rank, n),
            }
        })
        .collect()
}

fn build_introns(exons: &[Interval], strand: char) -> Vec<RankedInterval> {
    if exons.len() < 2 {
        return Vec::new();
    }

    let mut introns = Vec::new();
    for i in 0..(exons.len() - 1) {
        let left = &exons[i];
        let right = &exons[i + 1];
        if left.end < right.start {
            introns.push(Interval {
                start: left.end,
                end: right.start,
            });
        }
    }

    build_ranked_intervals(&introns, strand, "Intron")
}

fn parse_gtf_attributes(attrs: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for part in attrs.split(';') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut fields = trimmed.splitn(2, ' ');
        let key = match fields.next() {
            Some(v) if !v.is_empty() => v.trim(),
            _ => continue,
        };

        let value = match fields.next() {
            Some(v) => v.trim().trim_matches('"').to_string(),
            None => continue,
        };

        map.insert(key.to_string(), value);
    }
    map
}

#[derive(Debug, Clone)]
struct FeatureHit {
    annotation: GenomicAnnotation,
    gene_id: String,
    gene_symbol: String,
    transcript_id: String,
    distance_to_tss: i32,
    exon_intron_rank: String,
}

fn annotate_genomic_feature(genes: &GeneAnnotations, chr: &str, pos: u32) -> FeatureHit {
    let Some(transcripts) = genes.transcripts_by_chr.get(chr) else {
        return FeatureHit {
            annotation: GenomicAnnotation::Intergenic,
            gene_id: "".to_string(),
            gene_symbol: "".to_string(),
            transcript_id: "".to_string(),
            distance_to_tss: i32::MAX,
            exon_intron_rank: "".to_string(),
        };
    };

    let mut best_hit: Option<FeatureHit> = None;
    let mut nearest_intergenic: Option<FeatureHit> = None;

    for tx in transcripts {
        let distance_to_tss = signed_distance_to_tss(pos, tx.tss, tx.strand);
        let abs_dist = distance_to_tss.unsigned_abs();

        let update_nearest = |current: &mut Option<FeatureHit>| {
            let candidate = FeatureHit {
                annotation: GenomicAnnotation::Intergenic,
                gene_id: tx.gene_id.clone(),
                gene_symbol: tx.gene_symbol.clone(),
                transcript_id: tx.transcript_id.clone(),
                distance_to_tss,
                exon_intron_rank: "".to_string(),
            };

            let should_replace = match current {
                Some(existing) => abs_dist < existing.distance_to_tss.unsigned_abs(),
                None => true,
            };
            if should_replace {
                *current = Some(candidate);
            }
        };
        update_nearest(&mut nearest_intergenic);

        let hit = if tx.promoter.contains(pos) {
            Some(FeatureHit {
                annotation: GenomicAnnotation::Promoter,
                gene_id: tx.gene_id.clone(),
                gene_symbol: tx.gene_symbol.clone(),
                transcript_id: tx.transcript_id.clone(),
                distance_to_tss,
                exon_intron_rank: "".to_string(),
            })
        } else if tx.utr5.iter().any(|i| i.contains(pos)) {
            Some(FeatureHit {
                annotation: GenomicAnnotation::FivePrimeUtr,
                gene_id: tx.gene_id.clone(),
                gene_symbol: tx.gene_symbol.clone(),
                transcript_id: tx.transcript_id.clone(),
                distance_to_tss,
                exon_intron_rank: "".to_string(),
            })
        } else if tx.utr3.iter().any(|i| i.contains(pos)) {
            Some(FeatureHit {
                annotation: GenomicAnnotation::ThreePrimeUtr,
                gene_id: tx.gene_id.clone(),
                gene_symbol: tx.gene_symbol.clone(),
                transcript_id: tx.transcript_id.clone(),
                distance_to_tss,
                exon_intron_rank: "".to_string(),
            })
        } else if let Some(exon_hit) = tx.exons.iter().find(|e| e.interval.contains(pos)) {
            Some(FeatureHit {
                annotation: GenomicAnnotation::Exon,
                gene_id: tx.gene_id.clone(),
                gene_symbol: tx.gene_symbol.clone(),
                transcript_id: tx.transcript_id.clone(),
                distance_to_tss,
                exon_intron_rank: exon_hit.rank_label.clone(),
            })
        } else if let Some(intron_hit) = tx.introns.iter().find(|i| i.interval.contains(pos)) {
            Some(FeatureHit {
                annotation: GenomicAnnotation::Intron,
                gene_id: tx.gene_id.clone(),
                gene_symbol: tx.gene_symbol.clone(),
                transcript_id: tx.transcript_id.clone(),
                distance_to_tss,
                exon_intron_rank: intron_hit.rank_label.clone(),
            })
        } else if tx.downstream.contains(pos) {
            Some(FeatureHit {
                annotation: GenomicAnnotation::Downstream,
                gene_id: tx.gene_id.clone(),
                gene_symbol: tx.gene_symbol.clone(),
                transcript_id: tx.transcript_id.clone(),
                distance_to_tss,
                exon_intron_rank: "".to_string(),
            })
        } else {
            None
        };

        if let Some(candidate) = hit {
            let replace = match &best_hit {
                Some(existing) => compare_hits(&candidate, existing) == Ordering::Less,
                None => true,
            };

            if replace {
                best_hit = Some(candidate);
            }
        }
    }

    best_hit.or(nearest_intergenic).unwrap_or(FeatureHit {
        annotation: GenomicAnnotation::Intergenic,
        gene_id: "".to_string(),
        gene_symbol: "".to_string(),
        transcript_id: "".to_string(),
        distance_to_tss: i32::MAX,
        exon_intron_rank: "".to_string(),
    })
}

fn compare_hits(a: &FeatureHit, b: &FeatureHit) -> Ordering {
    a.annotation
        .priority()
        .cmp(&b.annotation.priority())
        .then_with(|| {
            a.distance_to_tss
                .unsigned_abs()
                .cmp(&b.distance_to_tss.unsigned_abs())
        })
}

fn signed_distance_to_tss(pos: u32, tss: u32, strand: char) -> i32 {
    if strand == '-' {
        tss as i32 - pos as i32
    } else {
        pos as i32 - tss as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promoter_priority_over_exon() {
        let tx = TranscriptModel {
            strand: '+',
            transcript_id: "tx1".to_string(),
            gene_id: "gene1".to_string(),
            gene_symbol: "G1".to_string(),
            tx_start: 1000,
            tx_end: 5000,
            tss: 1000,
            promoter: Interval {
                start: 0,
                end: 4000,
            },
            downstream: Interval {
                start: 5000,
                end: 8000,
            },
            exons: vec![RankedInterval {
                interval: Interval {
                    start: 1000,
                    end: 1500,
                },
                rank_label: "Exon 1 of 1".to_string(),
            }],
            introns: vec![],
            utr5: vec![],
            utr3: vec![],
        };

        let genes = GeneAnnotations {
            transcripts_by_chr: HashMap::from([("chr1".to_string(), vec![tx])]),
        };

        let hit = annotate_genomic_feature(&genes, "chr1", 1200);
        assert_eq!(hit.annotation, GenomicAnnotation::Promoter);
    }

    #[test]
    fn intergenic_uses_nearest_gene() {
        let tx = TranscriptModel {
            strand: '+',
            transcript_id: "tx2".to_string(),
            gene_id: "gene2".to_string(),
            gene_symbol: "G2".to_string(),
            tx_start: 100,
            tx_end: 200,
            tss: 100,
            promoter: Interval { start: 0, end: 50 },
            downstream: Interval {
                start: 200,
                end: 300,
            },
            exons: vec![],
            introns: vec![],
            utr5: vec![],
            utr3: vec![],
        };

        let genes = GeneAnnotations {
            transcripts_by_chr: HashMap::from([("chr2".to_string(), vec![tx])]),
        };

        let hit = annotate_genomic_feature(&genes, "chr2", 1000);
        assert_eq!(hit.annotation, GenomicAnnotation::Intergenic);
        assert_eq!(hit.gene_id, "gene2");
    }

    #[test]
    fn annotation_dir_is_required() {
        let err = load_annotation_resources("hg19", None).unwrap_err();
        assert!(
            err.to_string()
                .contains("Annotation requires --annotation-dir"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn sample_annotation_summary_is_coverage_aware() {
        let records = vec![
            AnnotationRecord {
                chr: "chr1".to_string(),
                start_1based: 101,
                end_1based: 102,
                strand: '+',
                annotation: "Promoter".to_string(),
                gene_id: "g1".to_string(),
                gene_symbol: "G1".to_string(),
                transcript_id: "tx1".to_string(),
                distance_to_tss: 0,
                exon_intron_rank: "".to_string(),
            },
            AnnotationRecord {
                chr: "chr1".to_string(),
                start_1based: 201,
                end_1based: 202,
                strand: '+',
                annotation: "Exon".to_string(),
                gene_id: "g1".to_string(),
                gene_symbol: "G1".to_string(),
                transcript_id: "tx1".to_string(),
                distance_to_tss: 100,
                exon_intron_rank: "Exon 1 of 1".to_string(),
            },
        ];

        let mut cov = Array2::<u32>::zeros((2, 2));
        cov[(0, 0)] = 5; // sample1 covers promoter
        cov[(1, 0)] = 0; // sample1 does not cover exon
        cov[(0, 1)] = 0; // sample2 does not cover promoter
        cov[(1, 1)] = 7; // sample2 covers exon

        let sample_names = vec!["sample1".to_string(), "sample2".to_string()];
        let summary = calculate_sample_annotation_summary(&records, &cov, &sample_names).unwrap();

        assert_eq!(summary.len(), 2);

        let s1 = summary.iter().find(|r| r.sample_name == "sample1").unwrap();
        assert_eq!(s1.covered_cpgs, 1);
        assert_eq!(s1.annotation_counts.get("Promoter"), Some(&1usize));
        assert_eq!(s1.annotation_counts.get("Exon"), None);

        let s2 = summary.iter().find(|r| r.sample_name == "sample2").unwrap();
        assert_eq!(s2.covered_cpgs, 1);
        assert_eq!(s2.annotation_counts.get("Exon"), Some(&1usize));
        assert_eq!(s2.annotation_counts.get("Promoter"), None);
    }
}
