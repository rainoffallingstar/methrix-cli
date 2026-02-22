#!/usr/bin/env Rscript

library(rhdf5)

assays_h5 <- "testdata/mCall/methrixh5/assays.h5"

cat("检查 assays.h5 结构\n")
cat("==========================================\n")

h5ls(assays_h5)

cat("\n\n/beta 维度和数据:\n")
beta <- h5read(assays_h5, "/beta")
cat("dim:", dim(beta), "\n")
cat("前6个值:\n")
print(head(beta))

cat("\n\n/cov 维度和数据:\n")
cov <- h5read(assays_h5, "/cov")
cat("dim:", dim(cov), "\n")
cat("前6个值:\n")
print(head(cov))
