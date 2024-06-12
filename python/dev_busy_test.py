#!/usr/bin/env python3
import os

fd = os.open("/dev/present80_encrypt", os.O_RDWR)
fd2 = os.open("/dev/present80_key", os.O_RDWR)

while True:
    pass
