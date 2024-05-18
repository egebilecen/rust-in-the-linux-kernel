import os
import secrets

data = secrets.token_urlsafe(16).encode("ascii") + b"\n"
print("Data    : {}".format(data))

fd = os.open("/dev/ee580", os.O_RDWR)
os.write(fd, data)

print("Response:", os.pread(fd, len(data), 0))
os.close(fd)
