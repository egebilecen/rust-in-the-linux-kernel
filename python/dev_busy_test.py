#!/usr/bin/env python3
import os

# Open the encrypt device.
fd = os.open("/dev/present80_encrypt", os.O_RDWR)
# Open the key device.
fd2 = os.open("/dev/present80_key", os.O_RDWR)

# Prevent the script from exiting so devices will be left open.
while True:
    pass
