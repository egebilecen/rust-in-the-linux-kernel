#!/usr/bin/env python3
from common import C_WORKING_DIR, RUST_WORKING_DIR, exec_cmd
from typing import Literal
import sys

# Parse the "objdump" result and return the size.
def get_module_size(_type: str, section: Literal["*"] | list[str] = "*", print_text: bool = False) -> int:
    cmd = ""
    cwd = ""
    _type = _type.lower()

    if _type == "rust":
        cmd = "objdump -t rust_misc_dev.ko --demangle"
        cwd = RUST_WORKING_DIR
    elif _type == "c":
        cmd = "objdump -t c_misc_dev.ko"
        cwd = C_WORKING_DIR

    res = exec_cmd(cmd, cwd, True)
    output = res[1].decode().split("\n")
    output = output[4:-3]

    total_size = 0

    for line in output:
        columns = {
            "address": 0,
            "flags": [],
            "section": "",
            "size": 0,
            "name": ""
        }
        _columns = line.split()
        flags_end_index = 0

        columns["address"] = int(_columns[0], 16)

        if _columns[1][0] not in [".", "*"]:
            for i, v in enumerate(_columns):
                if v[0] in [".", "*"]:
                    flags_end_index = i - 1
                    break

        if flags_end_index != 0:
            columns["flags"] = _columns[1:flags_end_index + 1]

        columns["section"] = _columns[flags_end_index + 1]
        columns["size"] = int(_columns[flags_end_index + 2], 16)
        columns["name"] = " ".join(_columns[flags_end_index + 3:])

        is_matching = False

        if section == "*":
            is_matching = True
        else:
            for val in section:
                if val in columns["section"]:
                    is_matching = True
                    break

        if is_matching and columns["size"] > 0:
            if print_text:
                if total_size == 0:
                    print("{: <32}{: <8}{}".format("[Section]", "[Size]", "[Name]"))
                print(" {: <32}{: <8}{}".format(columns["section"], columns["size"], columns["name"]))

            total_size += columns["size"]

    if print_text and total_size > 0:
        print()
        print("Total size: {}".format(total_size))
        print()

    return total_size

if len(sys.argv) > 1 and sys.argv[1] == "c":
    get_module_size("c", [".text"], True)
    get_module_size("c", [".data", ".rodata"], True)
    get_module_size("c", [".bss"], True)
    get_module_size("c", [".debug"], True)
    print("Total size of all sections in the module: {}".format(get_module_size("c", "*")))
elif len(sys.argv) > 1 and sys.argv[1] == "c-all":
    get_module_size("c", "*", True)
elif len(sys.argv) > 1 and sys.argv[1] == "rust":
    get_module_size("rust", [".text"], True)
    get_module_size("rust", [".data", ".rodata"], True)
    get_module_size("rust", [".bss"], True)
    get_module_size("rust", [".debug"], True)
    print("Total size of all sections in the module: {}".format(get_module_size("rust", "*")))
elif len(sys.argv) > 1 and sys.argv[1] == "rust-all":
    get_module_size("rust", "*", True)
else:
    print("Usage: ./module_size.py <c | rust | c-all | rust-all>")
    print("Example: ./module_size.py rust")
