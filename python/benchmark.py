import os
import time
from random import randbytes
from secrets import token_bytes
from typing import Any


def encrypt(key: bytes, plaintext: bytes) -> bytes:
    if len(key) != 10:
        raise ValueError(
            "This PRESENT-80 implementation only accepts a key of 10 bytes.")

    if len(plaintext) != 8:
        raise ValueError(
            "This PRESENT-80 implementation only accepts a data of 8 bytes.")

    key_fd = os.open("/dev/present80_key", os.O_RDWR)
    encryption_fd = os.open("/dev/present80_encrypt", os.O_RDWR)

    os.pwrite(key_fd, key, 0)
    os.pwrite(encryption_fd, plaintext, 0)

    res = os.pread(encryption_fd, len(plaintext), 0)

    os.close(key_fd)
    os.close(encryption_fd)

    return res


def printp(title: str = "", text: str = "") -> None:
    if title == "" and text == "":
        print()
        return

    title_width: int = 22
    title_adjusted: str = title.ljust(title_width)

    print(title_adjusted, end="")
    print(text, end="")
    print()


def get_time() -> float:
    return time.time_ns()


def get_time_result(time_ns: float) -> dict[str, Any]:
    return {
        "ns": time_ns,
        "us": time_ns / float(10 ** 3),
        "ms": time_ns / float(10 ** 6),
        "s": time_ns / float(10 ** 9)
    }


TOTAL_KEYS = 1000
TOTAL_PLAINTEXTS = 1000
TOTAL_ENCRYPTION = TOTAL_KEYS * TOTAL_PLAINTEXTS

key_list: list[bytes] = []
plaintext_list: list[bytes] = []

for _ in range(TOTAL_KEYS):
    key_list.append(token_bytes(10))

for _ in range(TOTAL_PLAINTEXTS):
    plaintext_list.append(randbytes(8))

start_time = get_time()
sum_encrypt_time = 0.0
end_time = 0.0

for key in key_list:
    for plaintext in plaintext_list:
        enc_start = get_time()
        encrypt(key, plaintext)
        enc_end = get_time()
        sum_encrypt_time += enc_end - enc_start

end_time = get_time()
time_diff_ns = end_time - start_time

time_result = get_time_result(time_diff_ns)
time_result["avg"] = get_time_result(sum_encrypt_time / TOTAL_ENCRYPTION)

printp("TOTAL KEYS", str(TOTAL_KEYS))
printp("TOTAL PLAINTEXTS", str(TOTAL_PLAINTEXTS))
printp("TOTAL ENCRYPTION", str(TOTAL_ENCRYPTION))
printp("TOTAL TIME",
       "{:.2f}sec / {:.2f}ms".format(time_result["s"], time_result["ms"]))
printp("AVG. ENCRYPTION TIME",
       "{:.2f}ns / {:.2f}us".format(time_result["avg"]["ns"],
                                    time_result["avg"]["us"]))
