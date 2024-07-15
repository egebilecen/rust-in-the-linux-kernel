#ifndef _C_MISC_DEV_H
#define _C_MISC_DEV_H

#include <linux/miscdevice.h>
#include <linux/fs.h>

/* Set the print format. */
#define pr_fmt(fmt) "%s: " fmt, KBUILD_MODNAME

/* Device prefix. */
#define DEVICE_PREFIX "present80"
/* Key device prefix. */
#define DEVICE_NAME_KEY DEVICE_PREFIX "_key"
/* Encryption device prefix. */
#define DEVICE_NAME_ENCRYPTION DEVICE_PREFIX "_encrypt"

/* Max. buffer size to allocate in stack. */
#define MAX_BUFFER_SIZE 10

/* Device type. */
enum misc_dev_type { KEY_DEVICE = 0, ENCRYPTION_DEVICE };

/* Device data. */
struct misc_dev_data {
	struct mutex lock;

	enum misc_dev_type type;
	bool is_in_use;

	u8 in_buffer[MAX_BUFFER_SIZE];
	u8 out_buffer[MAX_BUFFER_SIZE];
};

/* Device group. */
struct misc_dev_group {
	struct misc_dev_data key;
	struct misc_dev_data encryption;
};

#endif
