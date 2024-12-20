#!/usr/bin/env python3
import os

TEST_LIST = [
    # (<key>, <plain text>, <expected cipher text>)
    (
        b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        b"\x00\x00\x00\x00\x00\x00\x00\x00",
        bytes.fromhex("5579c1387b228445")
    ),
    (
        b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff",
        b"\x00\x00\x00\x00\x00\x00\x00\x00",
        bytes.fromhex("e72c46c0f5945049")
    ),
    (
        b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff",
        b"\xff\xff\xff\xff\xff\xff\xff\xff",
        bytes.fromhex("3333dcd3213210d2")
    ),
    (
        b"\xaa\xbb\xcc\xdd\xee\xff\x11\x22\x33\x44",
        b"\xde\xad\xbe\xef\xde\xad\xc0\xde",
        bytes.fromhex("31f69aa9604e985f")
    )
]

print("Total tests: {}\n".format(len(TEST_LIST)))

# Open the key device.
key_fd = os.open("/dev/present80_key", os.O_RDWR)
# Open the encryption device.
encryption_fd = os.open("/dev/present80_encrypt", os.O_RDWR)

# Loop over each entry.
for (i, (key, plain_text, cipher_text)) in enumerate(TEST_LIST):
    print("[Test {}]".format(i + 1))
    print("Key:\t\t{}".format(key.hex()))
    print("Plaintext:\t{}".format(plain_text.hex()))
    print("Ciphertext:\t{}".format(cipher_text.hex()))

    # Set the key.
    os.pwrite(key_fd, key, 0)
    # Encrypt the plaintext.
    os.pwrite(encryption_fd, plain_text, 0)

    # Read the encryption result.
    res = os.pread(encryption_fd, len(plain_text), 0)

    print("Result:\t\t{}".format(res.hex()))
    print()

    if res == cipher_text:
        print("TEST SUCCESS!")
    else:
        print("TEST FAILED!")

    print()

# Close the devices.
os.close(key_fd)
os.close(encryption_fd)
