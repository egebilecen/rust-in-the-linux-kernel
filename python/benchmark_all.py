#!/usr/bin/env python3
import json
import csv
from common import C_WORKING_DIR, RUST_WORKING_DIR, exec_cmd

def format_float(num):
    return "{:.2f}".format(num)

# Number of benchmarks.
TOTAL_BENCHMARKS = 5

# Output results file.
RESULTS_FILE = "result_{}.csv"
RESULTS_C_AVG_ENCRYPTION_TIME = []
RESULTS_RUST_AVG_ENCRYPTION_TIME = []

RESULTS_C_TOTAL_TIME = []
RESULTS_RUST_TOTAL_TIME = []

print("Benchmarking C...")
exec_cmd("./run", C_WORKING_DIR)

# Benchmark the C module by running the "benchmark.py".
for i in range(TOTAL_BENCHMARKS):
    res = exec_cmd("./benchmark.py json".split())
    json_str = res[1].decode()
    json_obj = json.loads(json_str)

    RESULTS_C_AVG_ENCRYPTION_TIME.append(format_float(json_obj["avg_encryption_time"]["us"]))
    RESULTS_C_TOTAL_TIME.append(format_float(json_obj["total_time"]["s"]))

    print("Benchmark #{} completed.".format(i + 1))

print()

print("Benchmarking Rust...")
exec_cmd("./run", RUST_WORKING_DIR)

# Benchmark the Rust module by running the "benchmark.py".
for i in range(TOTAL_BENCHMARKS):
    res = exec_cmd("./benchmark.py json".split())
    json_str = res[1].decode()
    json_obj = json.loads(json_str)

    RESULTS_RUST_AVG_ENCRYPTION_TIME.append(format_float(json_obj["avg_encryption_time"]["us"]))
    RESULTS_RUST_TOTAL_TIME.append(format_float(json_obj["total_time"]["s"]))

    print("Benchmark #{} completed.".format(i + 1))

common_row = ["benchmark_no", "C", "Rust"]

# Save the results.
with open(RESULTS_FILE.format("avg_enc_time_us"), "w") as f:
    writer = csv.writer(f)
    writer.writerow(common_row)
    writer.writerows([["#{}".format(i + 1), RESULTS_C_AVG_ENCRYPTION_TIME[i], RESULTS_RUST_AVG_ENCRYPTION_TIME[i]] for i in range(TOTAL_BENCHMARKS)])

with open(RESULTS_FILE.format("total_enc_time_s"), "w") as f:
    writer = csv.writer(f)
    writer.writerow(common_row)
    writer.writerows([["#{}".format(i + 1), RESULTS_C_TOTAL_TIME[i], RESULTS_RUST_TOTAL_TIME[i]] for i in range(TOTAL_BENCHMARKS)])

print("Results are written into the related CSV files.")
