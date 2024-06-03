#!/usr/bin/env python3
import subprocess
import json
import csv

def exec_cmd(cmd, cwd=".", shell=False):
    proc = subprocess.Popen(cmd, cwd=cwd, shell=shell, stderr=subprocess.PIPE, stdout=subprocess.PIPE)
    outs, errs = proc.communicate()
    return (proc.returncode, outs, errs)

def format_float(num):
    return "{:.2f}".format(num)

TOTAL_BENCHMARKS = 5
C_WORKING_DIR = "../c/"
RUST_WORKING_DIR = "../rust/"

RESULTS_FILE = "result_{}.csv"

RESULTS_C_AVG_ENCRYPTION_TIME = []
RESULTS_RUST_AVG_ENCRYPTION_TIME = []

RESULTS_C_TOTAL_TIME = []
RESULTS_RUST_TOTAL_TIME = []

print("Benchmarking C...")
exec_cmd("./run", C_WORKING_DIR)

for i in range(TOTAL_BENCHMARKS):
    res = exec_cmd("./benchmark.py json".split())
    json_str = res[1].decode()
    json_obj = json.loads(json_str)

    RESULTS_C_AVG_ENCRYPTION_TIME.append(format_float(json_obj["avg_encryption_time"]["us"]))
    RESULTS_C_TOTAL_TIME.append(format_float(json_obj["total_time"]["s"]))

print("Benchmarking Rust...")
exec_cmd("./run", RUST_WORKING_DIR)

for i in range(TOTAL_BENCHMARKS):
    res = exec_cmd("./benchmark.py json".split())
    json_str = res[1].decode()
    json_obj = json.loads(json_str)

    RESULTS_RUST_AVG_ENCRYPTION_TIME.append(format_float(json_obj["avg_encryption_time"]["us"]))
    RESULTS_RUST_TOTAL_TIME.append(format_float(json_obj["total_time"]["s"]))

common_row = ["benchmark_no", "C", "Rust"]

with open(RESULTS_FILE.format("avg_enc_time_us"), "w") as f:
    writer = csv.writer(f)
    writer.writerow(common_row)
    writer.writerows([["#{}".format(i + 1), RESULTS_C_AVG_ENCRYPTION_TIME[i], RESULTS_RUST_AVG_ENCRYPTION_TIME[i]] for i in range(TOTAL_BENCHMARKS)])

with open(RESULTS_FILE.format("total_enc_time_s"), "w") as f:
    writer = csv.writer(f)
    writer.writerow(common_row)
    writer.writerows([["#{}".format(i + 1), RESULTS_C_TOTAL_TIME[i], RESULTS_RUST_TOTAL_TIME[i]] for i in range(TOTAL_BENCHMARKS)])

print("Results are written into the related CSV files.")
