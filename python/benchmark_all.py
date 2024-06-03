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
RESULTS_FILE = "results.csv"
TIME_FORMAT = "us"

C_WORKING_DIR = "../c/"
C_BENCHMARK_RESULTS = []

RUST_WORKING_DIR = "../rust/"
RUST_BENCHMARK_RESULTS = []

print("Benchmarking C...")
exec_cmd("./run", C_WORKING_DIR)

for i in range(TOTAL_BENCHMARKS):
    res = exec_cmd("./benchmark.py json".split())
    json_str = res[1].decode()
    json_obj = json.loads(json_str)

    C_BENCHMARK_RESULTS.append(format_float(json_obj["avg_encryption_time"][TIME_FORMAT]))

print("Benchmarking Rust...")
exec_cmd("./run", RUST_WORKING_DIR)

for i in range(TOTAL_BENCHMARKS):
    res = exec_cmd("./benchmark.py json".split())
    json_str = res[1].decode()
    json_obj = json.loads(json_str)

    RUST_BENCHMARK_RESULTS.append(format_float(json_obj["avg_encryption_time"][TIME_FORMAT]))

with open(RESULTS_FILE, "w") as f:
    writer = csv.writer(f)
    writer.writerows([
        ["benchmark_no", "C", "Rust"]
    ] + [["#{}".format(i + 1), C_BENCHMARK_RESULTS[i], RUST_BENCHMARK_RESULTS[i]] for i in range(TOTAL_BENCHMARKS)])

print("Results are written into the \"{}\" file.".format(RESULTS_FILE))
