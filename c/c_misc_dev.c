#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/printk.h>

#define pr_fmt(fmt) "%s: " fmt, KBUILD_MODNAME

static int mod_init(void) {
  pr_info("Initializing...\n");
  return 0;
}

static void mod_exit(void) { pr_info("Exited...\n"); }

module_init(mod_init);
module_exit(mod_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Ege Bilecen");
MODULE_DESCRIPTION("Miscellaneous device written in C.");
