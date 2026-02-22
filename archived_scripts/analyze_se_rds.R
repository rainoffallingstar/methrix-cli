#!/usr/bin/env Rscript

library(methrix)

se_file <- "/public3/home/scg9946/methrix-cli/testdata/mCall/methrixh5/se.rds"

cat("分析 R methrix 的 se.rds 结构\n")
cat("==========================================\n\n")

se <- readRDS(se_file)

cat("对象类型:", class(se)[1], "\n")
cat("维度:", nrow(se), "x", ncol(se), "\n\n")

cat("assays:\n")
print(names(assays(se)))

cat("\nrowData:\n")
cat("  列名:", colnames(rowData(se)), "\n")
cat("  行数:", nrow(rowData(se)), "\n")
cat("  前3行:\n")
print(head(rowData(se), 3))

cat("\ncolData:\n")
cat("  列名:", colnames(colData(se)), "\n")
cat("  样本数:", nrow(colData(se)), "\n")
print(colData(se))

cat("\nmetadata:\n")
print(names(metadata(se)))
cat("  genome:", metadata(se)$genome, "\n")
cat("  is_h5:", metadata(se)$is_h5, "\n")

cat("\n@assays@elementMetadata:\n")
if (!is.null(S4Vectors::mcols(assays(se)))) {
  print(S4Vectors::mcols(assays(se)))
}
