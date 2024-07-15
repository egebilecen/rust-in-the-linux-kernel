#!/usr/bin/env python3
import subprocess

C_WORKING_DIR = "../c/"
RUST_WORKING_DIR = "../rust/"

# Execute a (shell) command.
def exec_cmd(cmd, cwd=".", shell=False):
    proc = subprocess.Popen(cmd, cwd=cwd, shell=shell, stderr=subprocess.PIPE, stdout=subprocess.PIPE)
    outs, errs = proc.communicate()
    return (proc.returncode, outs, errs)
