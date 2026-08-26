use anyhow::{Context, Result};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use ndarray::Array2;
use rayon::prelude::*;
use rust_xlsxwriter::Workbook;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::atomic_output::{write_atomically, AtomicOutputSet};
use crate::cli::process::canonical_contig_name;
use crate::genome::cpg::CpGSite;

const PROMOTER_WINDOW: u32 = 3_000;
const DOWNSTREAM_WINDOW: u32 = 3_000;
const QCTB_REQUIRED_ANNOTATIONS: [&str; 4] = ["Promoter", "Exon", "Intron", "Intergenic"];

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
struct FeatureInterval {
    start: u32,
    end: u32,
    annotation: GenomicAnnotation,
    rank_label: Option<String>,
    tx_idx: usize,
}

#[derive(Debug, Clone)]
struct ChromosomeAnnotationIndex {
    transcripts: Vec<TranscriptModel>,
    tss_sorted: Vec<(u32, usize)>,
    buckets: Vec<Vec<FeatureInterval>>,
}

#[derive(Debug, Clone)]
struct GeneAnnotations {
    indices_by_chr: HashMap<String, ChromosomeAnnotationIndex>,
}

impl GeneAnnotations {
    fn from_transcripts(transcripts_by_chr: HashMap<String, Vec<TranscriptModel>>) -> Self {
        let mut canonical_transcripts: HashMap<String, Vec<TranscriptModel>> = HashMap::new();
        for (chr, txs) in transcripts_by_chr {
            canonical_transcripts
                .entry(canonical_contig_name(&chr))
                .or_default()
                .extend(txs);
        }

        let mut indices_by_chr = HashMap::new();
        for (canon_chr, mut txs) in canonical_transcripts.clone() {
            txs.sort_by(|a, b| {
                a.tx_start
                    .cmp(&b.tx_start)
                    .then_with(|| a.tx_end.cmp(&b.tx_end))
            });

            let mut tss_sorted: Vec<(u32, usize)> = txs
                .iter()
                .enumerate()
                .map(|(idx, tx)| (tx.tss, idx))
                .collect();
            tss_sorted.sort_by_key(|(tss, _)| *tss);

            let mut max_coord = 0u32;
            let mut intervals = Vec::new();
            for (tx_idx, tx) in txs.iter().enumerate() {
                intervals.push(FeatureInterval {
                    start: tx.promoter.start,
                    end: tx.promoter.end,
                    annotation: GenomicAnnotation::Promoter,
                    rank_label: None,
                    tx_idx,
                });
                max_coord = max_coord.max(tx.promoter.end);

                for u in &tx.utr5 {
                    intervals.push(FeatureInterval {
                        start: u.start,
                        end: u.end,
                        annotation: GenomicAnnotation::FivePrimeUtr,
                        rank_label: None,
                        tx_idx,
                    });
                    max_coord = max_coord.max(u.end);
                }
                for u in &tx.utr3 {
                    intervals.push(FeatureInterval {
                        start: u.start,
                        end: u.end,
                        annotation: GenomicAnnotation::ThreePrimeUtr,
                        rank_label: None,
                        tx_idx,
                    });
                    max_coord = max_coord.max(u.end);
                }
                for e in &tx.exons {
                    intervals.push(FeatureInterval {
                        start: e.interval.start,
                        end: e.interval.end,
                        annotation: GenomicAnnotation::Exon,
                        rank_label: Some(e.rank_label.clone()),
                        tx_idx,
                    });
                    max_coord = max_coord.max(e.interval.end);
                }
                for i in &tx.introns {
                    intervals.push(FeatureInterval {
                        start: i.interval.start,
                        end: i.interval.end,
                        annotation: GenomicAnnotation::Intron,
                        rank_label: Some(i.rank_label.clone()),
                        tx_idx,
                    });
                    max_coord = max_coord.max(i.interval.end);
                }
                intervals.push(FeatureInterval {
                    start: tx.downstream.start,
                    end: tx.downstream.end,
                    annotation: GenomicAnnotation::Downstream,
                    rank_label: None,
                    tx_idx,
                });
                max_coord = max_coord.max(tx.downstream.end);
            }

            let num_buckets = ((max_coord >> 16) as usize) + 2;
            let mut buckets = vec![Vec::new(); num_buckets];
            for interval in intervals {
                let start_bucket = (interval.start >> 16) as usize;
                let end_bucket = if interval.end > 0 {
                    ((interval.end - 1) >> 16) as usize
                } else {
                    start_bucket
                };
                for b in start_bucket..=end_bucket.min(num_buckets - 1) {
                    buckets[b].push(interval.clone());
                }
            }

            indices_by_chr.insert(
                canon_chr,
                ChromosomeAnnotationIndex {
                    transcripts: txs,
                    tss_sorted,
                    buckets,
                },
            );
        }

        Self { indices_by_chr }
    }
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
        write_atomically(Path::new(output_path), |temporary_path| {
            self.write_excel_report_to_path(temporary_path)
        })
    }

    pub(crate) fn write_excel_report_to_path(&self, output_path: &Path) -> Result<()> {
        let mut workbook = Workbook::new();

        let _sheet_by_sample = workbook.add_worksheet();
        let sheet_by_sample = workbook
            .worksheet_from_index(0)
            .context("Failed to get by-sample worksheet")?;
        sheet_by_sample.set_name("ChIPseeker_By_Sample")?;

        let annotations = self.report_annotations();

        sheet_by_sample.write_string(0, 0, "sample")?;
        sheet_by_sample.write_string(0, 1, "covered_cpgs")?;
        let mut col = 2u16;
        for ann in &annotations {
            sheet_by_sample.write_string(0, col, format!("{}_count", ann))?;
            sheet_by_sample.write_string(0, col + 1, format!("{}_percent", ann))?;
            col += 2;
        }

        for (sample_index, sample_row) in self.sample_summary.iter().enumerate() {
            let row = (sample_index + 1) as u32;
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

        workbook.save(output_path)?;
        Ok(())
    }

    pub fn write_report_set(&self, summary_path: &str, details_path: &str) -> Result<()> {
        let summary_path = Path::new(summary_path);
        let details_path = Path::new(details_path);
        let output_directory = summary_path.parent().unwrap_or_else(|| Path::new("."));
        let mut output_set = AtomicOutputSet::new(output_directory)?;
        output_set.stage(summary_path, |temporary_path| {
            self.write_excel_report_to_path(temporary_path)
        })?;
        output_set.stage(details_path, |temporary_path| {
            self.write_details_to_path(temporary_path)
        })?;
        output_set.publish()
    }

    pub(crate) fn write_details_to_path(&self, output_path: &Path) -> Result<()> {
        let output_file = File::create(output_path)
            .with_context(|| format!("Failed to create {}", output_path.display()))?;
        let encoder = GzEncoder::new(output_file, Compression::default());
        let mut writer = std::io::BufWriter::new(encoder);
        writeln!(
            writer,
            "chr\tstart_1based\tend_1based\tstrand\tannotation\tgene_id\tgene_symbol\ttranscript_id\tdistance_to_tss\texon_intron_rank"
        )?;
        for record in &self.records {
            let values = [
                record.chr.as_str(),
                &record.start_1based.to_string(),
                &record.end_1based.to_string(),
                &record.strand.to_string(),
                record.annotation.as_str(),
                record.gene_id.as_str(),
                record.gene_symbol.as_str(),
                record.transcript_id.as_str(),
                &record.distance_to_tss.to_string(),
                record.exon_intron_rank.as_str(),
            ];
            writeln!(
                writer,
                "{}",
                values
                    .iter()
                    .map(|value| escape_tsv_field(value))
                    .collect::<Vec<_>>()
                    .join("\t")
            )?;
        }
        writer.flush()?;
        let encoder = writer.into_inner().map_err(|error| error.into_error())?;
        encoder.finish()?.sync_all()?;
        Ok(())
    }

    fn report_annotations(&self) -> Vec<String> {
        let mut extras: Vec<String> = self
            .records
            .iter()
            .map(|record| record.annotation.clone())
            .filter(|annotation| !QCTB_REQUIRED_ANNOTATIONS.contains(&annotation.as_str()))
            .collect();
        extras.sort();
        extras.dedup();

        QCTB_REQUIRED_ANNOTATIONS
            .iter()
            .map(|annotation| (*annotation).to_string())
            .chain(extras)
            .collect()
    }
}

fn escape_tsv_field(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\t' | '\n' | '\r' => ' ',
            other => other,
        })
        .collect()
}

pub fn annotate_cpgs(
    cpg_sites: &[CpGSite],
    cov_matrix: &Array2<u32>,
    sample_names: &[String],
    genome: &str,
    annotation_dir: Option<&str>,
) -> Result<AnnotationResult> {
    let resources = load_annotation_resources(genome, annotation_dir)?;

    let records: Vec<AnnotationRecord> = cpg_sites
        .par_iter()
        .map(|cpg| {
            let chip = annotate_genomic_feature(&resources.genes, &cpg.chr, cpg.start);
            AnnotationRecord {
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
            }
        })
        .collect();

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

        let chr = canonical_contig_name(fields[0]);
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

    Ok(GeneAnnotations::from_transcripts(transcripts_by_chr))
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
    let canonical_chr = canonical_contig_name(chr);
    let Some(index) = genes.indices_by_chr.get(&canonical_chr) else {
        return FeatureHit {
            annotation: GenomicAnnotation::Intergenic,
            gene_id: "".to_string(),
            gene_symbol: "".to_string(),
            transcript_id: "".to_string(),
            distance_to_tss: i32::MAX,
            exon_intron_rank: "".to_string(),
        };
    };

    let mut best_hit_ann: Option<GenomicAnnotation> = None;
    let mut best_hit_dist: i32 = 0;
    let mut best_hit_rank: Option<&str> = None;
    let mut best_hit_idx: Option<usize> = None;

    let bucket_idx = (pos >> 16) as usize;
    if let Some(bucket) = index.buckets.get(bucket_idx) {
        for item in bucket {
            if item.start <= pos && pos < item.end {
                let tx = &index.transcripts[item.tx_idx];
                let distance_to_tss = signed_distance_to_tss(pos, tx.tss, tx.strand);
                let abs_dist = distance_to_tss.unsigned_abs();

                let replace = match best_hit_ann {
                    Some(existing_ann) => {
                        let p_cand = item.annotation.priority();
                        let p_exist = existing_ann.priority();
                        if p_cand < p_exist {
                            true
                        } else if p_cand == p_exist {
                            abs_dist < best_hit_dist.unsigned_abs()
                        } else {
                            false
                        }
                    }
                    None => true,
                };

                if replace {
                    best_hit_ann = Some(item.annotation);
                    best_hit_dist = distance_to_tss;
                    best_hit_rank = item.rank_label.as_deref();
                    best_hit_idx = Some(item.tx_idx);
                }
            }
        }
    }

    if let (Some(ann), Some(idx)) = (best_hit_ann, best_hit_idx) {
        let tx = &index.transcripts[idx];
        FeatureHit {
            annotation: ann,
            gene_id: tx.gene_id.clone(),
            gene_symbol: tx.gene_symbol.clone(),
            transcript_id: tx.transcript_id.clone(),
            distance_to_tss: best_hit_dist,
            exon_intron_rank: best_hit_rank.unwrap_or_default().to_string(),
        }
    } else if !index.tss_sorted.is_empty() {
        let pivot = index.tss_sorted.partition_point(|(tss, _)| *tss < pos);
        let mut min_abs = u32::MAX;
        let mut chosen_idx = 0;
        let mut chosen_dist = i32::MAX;

        let start_range = pivot.saturating_sub(1);
        let end_range = (pivot + 2).min(index.tss_sorted.len());
        for i in start_range..end_range {
            let (_, tx_idx) = index.tss_sorted[i];
            let tx = &index.transcripts[tx_idx];
            let dist = signed_distance_to_tss(pos, tx.tss, tx.strand);
            let abs_d = dist.unsigned_abs();
            if abs_d < min_abs {
                min_abs = abs_d;
                chosen_idx = tx_idx;
                chosen_dist = dist;
            }
        }

        let tx = &index.transcripts[chosen_idx];
        FeatureHit {
            annotation: GenomicAnnotation::Intergenic,
            gene_id: tx.gene_id.clone(),
            gene_symbol: tx.gene_symbol.clone(),
            transcript_id: tx.transcript_id.clone(),
            distance_to_tss: chosen_dist,
            exon_intron_rank: "".to_string(),
        }
    } else {
        FeatureHit {
            annotation: GenomicAnnotation::Intergenic,
            gene_id: "".to_string(),
            gene_symbol: "".to_string(),
            transcript_id: "".to_string(),
            distance_to_tss: i32::MAX,
            exon_intron_rank: "".to_string(),
        }
    }
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn report_always_contains_qctb_required_annotation_columns() {
        let result = AnnotationResult {
            records: vec![AnnotationRecord {
                chr: "chr1".to_string(),
                start_1based: 10,
                end_1based: 11,
                strand: '+',
                annotation: "5' UTR".to_string(),
                gene_id: String::new(),
                gene_symbol: String::new(),
                transcript_id: String::new(),
                distance_to_tss: 0,
                exon_intron_rank: String::new(),
            }],
            sample_summary: Vec::new(),
        };

        assert_eq!(
            result.report_annotations(),
            vec!["Promoter", "Exon", "Intron", "Intergenic", "5' UTR"]
        );
    }

    #[test]
    fn writes_unbounded_annotation_details_as_gzip_tsv() {
        let directory = tempfile::tempdir().unwrap();
        let details_path = directory.path().join("details.tsv.gz");
        let result = AnnotationResult {
            records: vec![AnnotationRecord {
                chr: "chr1".to_string(),
                start_1based: 10,
                end_1based: 11,
                strand: '+',
                annotation: "Promoter".to_string(),
                gene_id: "gene\t1".to_string(),
                gene_symbol: "G1".to_string(),
                transcript_id: "tx1".to_string(),
                distance_to_tss: -5,
                exon_intron_rank: String::new(),
            }],
            sample_summary: Vec::new(),
        };

        result.write_details_to_path(&details_path).unwrap();
        let file = File::open(details_path).unwrap();
        let mut reader = std::io::BufReader::new(GzDecoder::new(file));
        let mut decoded = String::new();
        std::io::Read::read_to_string(&mut reader, &mut decoded).unwrap();
        assert!(decoded.starts_with("chr\tstart_1based\tend_1based"));
        assert!(decoded.contains("gene 1"));
    }

    #[test]
    fn gtf_chr_aliases_match_cpg_contig_names() -> Result<()> {
        let mut gtf_file = NamedTempFile::new()?;
        writeln!(
            gtf_file,
            "chr1\ttest\ttranscript\t101\t200\t.\t+\t.\tgene_id \"gene_chr1\"; transcript_id \"tx_chr1\"; gene_name \"GENE1\";"
        )?;
        writeln!(
            gtf_file,
            "chr1\ttest\texon\t101\t200\t.\t+\t.\tgene_id \"gene_chr1\"; transcript_id \"tx_chr1\"; gene_name \"GENE1\";"
        )?;
        writeln!(
            gtf_file,
            "chrM\ttest\ttranscript\t501\t600\t.\t+\t.\tgene_id \"gene_chr_m\"; transcript_id \"tx_chr_m\"; gene_name \"MTGENE\";"
        )?;
        writeln!(
            gtf_file,
            "chrM\ttest\texon\t501\t600\t.\t+\t.\tgene_id \"gene_chr_m\"; transcript_id \"tx_chr_m\"; gene_name \"MTGENE\";"
        )?;

        let genes = load_genes_from_gtf(gtf_file.path())?;

        let autosome_hit = annotate_genomic_feature(&genes, "1", 150);
        assert_eq!(autosome_hit.gene_id, "gene_chr1");
        assert_ne!(autosome_hit.annotation, GenomicAnnotation::Intergenic);

        let mitochondrial_hit = annotate_genomic_feature(&genes, "MT", 550);
        assert_eq!(mitochondrial_hit.gene_id, "gene_chr_m");
        assert_ne!(mitochondrial_hit.annotation, GenomicAnnotation::Intergenic);

        Ok(())
    }

    #[test]
    fn gtf_contig_aliases_match_cpg_contigs() -> Result<()> {
        let mut gtf_file = NamedTempFile::new()?;
        writeln!(
            gtf_file,
            "chr1\ttest\ttranscript\t10001\t10100\t.\t+\t.\tgene_id \"gene_chr1\"; transcript_id \"tx_chr1\"; gene_name \"GENE1\";"
        )?;
        writeln!(
            gtf_file,
            "chr1\ttest\texon\t10001\t10100\t.\t+\t.\tgene_id \"gene_chr1\"; transcript_id \"tx_chr1\"; gene_name \"GENE1\";"
        )?;
        writeln!(
            gtf_file,
            "chrM\ttest\ttranscript\t201\t300\t.\t+\t.\tgene_id \"gene_mito\"; transcript_id \"tx_mito\"; gene_name \"MTGENE\";"
        )?;
        writeln!(
            gtf_file,
            "chrM\ttest\texon\t201\t300\t.\t+\t.\tgene_id \"gene_mito\"; transcript_id \"tx_mito\"; gene_name \"MTGENE\";"
        )?;
        gtf_file.flush()?;

        let genes = load_genes_from_gtf(gtf_file.path())?;

        let chromosome_hit = annotate_genomic_feature(&genes, "1", 10_049);
        assert_eq!(chromosome_hit.gene_id, "gene_chr1");
        assert_ne!(chromosome_hit.annotation, GenomicAnnotation::Intergenic);

        let mitochondrial_hit = annotate_genomic_feature(&genes, "MT", 249);
        assert_eq!(mitochondrial_hit.gene_id, "gene_mito");
        assert_ne!(mitochondrial_hit.annotation, GenomicAnnotation::Intergenic);

        Ok(())
    }

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

        let genes = GeneAnnotations::from_transcripts(HashMap::from([("1".to_string(), vec![tx])]));

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

        let genes = GeneAnnotations::from_transcripts(HashMap::from([("2".to_string(), vec![tx])]));

        let hit = annotate_genomic_feature(&genes, "chr2", 1000);
        assert_eq!(hit.annotation, GenomicAnnotation::Intergenic);
        assert_eq!(hit.gene_id, "gene2");
    }

    #[test]
    fn classifies_all_supported_transcript_regions() {
        let transcript = TranscriptModel {
            strand: '+',
            transcript_id: "tx_regions".to_string(),
            gene_id: "gene_regions".to_string(),
            gene_symbol: "REGIONS".to_string(),
            tx_start: 100,
            tx_end: 500,
            tss: 100,
            promoter: Interval {
                start: 90,
                end: 110,
            },
            downstream: Interval {
                start: 500,
                end: 600,
            },
            exons: vec![
                RankedInterval {
                    interval: Interval {
                        start: 120,
                        end: 150,
                    },
                    rank_label: "Exon 1 of 2".to_string(),
                },
                RankedInterval {
                    interval: Interval {
                        start: 200,
                        end: 230,
                    },
                    rank_label: "Exon 2 of 2".to_string(),
                },
            ],
            introns: vec![RankedInterval {
                interval: Interval {
                    start: 150,
                    end: 200,
                },
                rank_label: "Intron 1 of 1".to_string(),
            }],
            utr5: vec![Interval {
                start: 110,
                end: 120,
            }],
            utr3: vec![Interval {
                start: 230,
                end: 250,
            }],
        };
        let genes =
            GeneAnnotations::from_transcripts(HashMap::from([("1".to_string(), vec![transcript])]));

        let expected_annotations = [
            (100, GenomicAnnotation::Promoter, ""),
            (115, GenomicAnnotation::FivePrimeUtr, ""),
            (130, GenomicAnnotation::Exon, "Exon 1 of 2"),
            (175, GenomicAnnotation::Intron, "Intron 1 of 1"),
            (235, GenomicAnnotation::ThreePrimeUtr, ""),
            (550, GenomicAnnotation::Downstream, ""),
            (700, GenomicAnnotation::Intergenic, ""),
        ];

        for (position, expected_annotation, expected_rank) in expected_annotations {
            let hit = annotate_genomic_feature(&genes, "chr1", position);
            assert_eq!(hit.annotation, expected_annotation, "position {position}");
            assert_eq!(hit.exon_intron_rank, expected_rank, "position {position}");
            assert_eq!(hit.gene_id, "gene_regions", "position {position}");
        }
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
    fn annotation_summary_rejects_dimension_mismatch() {
        let records = vec![AnnotationRecord {
            chr: "chr1".to_string(),
            start_1based: 101,
            end_1based: 102,
            strand: '+',
            annotation: "Promoter".to_string(),
            gene_id: "g1".to_string(),
            gene_symbol: "G1".to_string(),
            transcript_id: "tx1".to_string(),
            distance_to_tss: 0,
            exon_intron_rank: String::new(),
        }];
        let coverage = Array2::<u32>::zeros((2, 1));
        let sample_names = vec!["sample1".to_string()];

        let error =
            calculate_sample_annotation_summary(&records, &coverage, &sample_names).unwrap_err();
        assert!(error.to_string().contains("dimensions"));
        assert!(error.to_string().contains("1 records"));
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
