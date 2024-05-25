Be sure to build the module with LLVM activated, ie, `make LLVM=1`. LSP can be enabled by going to kernel directory and running the following command: `scripts/clang-tools/gen_compile_commands.py`. This will generate `compile_commands.json`. Copy this file to the kernel module root folder.

