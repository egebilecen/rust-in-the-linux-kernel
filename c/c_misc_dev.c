#include <linux/miscdevice.h>
#include <linux/fs.h>

#define pr_fmt(fmt) "%s: " fmt, KBUILD_MODNAME

#define DEVICE_PREFIX "present80"
#define DEVICE_NAME_KEY DEVICE_PREFIX "_key"
#define DEVICE_NAME_ENCRYPTION DEVICE_PREFIX "_encrypt"

#define BUFFER_SIZE 10 /* In bytes. */

struct misc_dev_data {
	atomic_t is_in_use;
	u8 in_buffer[BUFFER_SIZE];
	u8 out_buffer[BUFFER_SIZE];
};

struct misc_dev_group {
	struct misc_dev_data key;
	struct misc_dev_data encryption;
};

/*///////////////////////////////////////////////////////////////////////////*/

static int dev_open(struct inode *, struct file *);
static ssize_t dev_read(struct file *, char __user *, size_t, loff_t *);
static ssize_t dev_write(struct file *, const char __user *, size_t, loff_t *);
static int dev_release(struct inode *, struct file *);

/*///////////////////////////////////////////////////////////////////////////*/

static struct file_operations dev_fops = { .open = dev_open,
					   .read = dev_read,
					   .write = dev_write,
					   .release = dev_release };

static struct miscdevice key_dev = { .minor = MISC_DYNAMIC_MINOR,
				     .name = DEVICE_NAME_KEY,
				     .fops = &dev_fops };

static struct miscdevice encryption_dev = { .minor = MISC_DYNAMIC_MINOR,
					    .name = DEVICE_NAME_ENCRYPTION,
					    .fops = &dev_fops };

static struct misc_dev_group dev_group;

/*///////////////////////////////////////////////////////////////////////////*/

static struct misc_dev_data *get_misc_dev_data(struct file *file)
{
	int minor = iminor(file->f_inode);

	return minor == key_dev.minor	     ? &dev_group.key :
	       minor == encryption_dev.minor ? &dev_group.encryption :
					       NULL;
}

static void init_misc_dev_group(struct misc_dev_group *group)
{
	group->key.is_in_use.counter = 0;
	memset(group->key.in_buffer, 0, sizeof(group->key.in_buffer));
	memset(group->key.out_buffer, 0, sizeof(group->key.out_buffer));

	group->encryption.is_in_use.counter = 0;
	memset(group->encryption.in_buffer, 0,
	       sizeof(group->encryption.in_buffer));
	memset(group->encryption.out_buffer, 0,
	       sizeof(group->encryption.out_buffer));
}

static int dev_open(struct inode *inode, struct file *file)
{
	struct misc_dev_data *dev_data = get_misc_dev_data(file);

	if (atomic_cmpxchg(&dev_data->is_in_use, 0, 1))
		return -EBUSY;

	return 0;
}

static ssize_t dev_read(struct file *file, char __user *buff, size_t len,
			loff_t *offset)
{
	struct misc_dev_data *dev_data = get_misc_dev_data(file);

	return -EPERM;
}

static ssize_t dev_write(struct file *file, const char __user *buff, size_t len,
			 loff_t *offset)
{
	struct misc_dev_data *dev_data = get_misc_dev_data(file);

	return -EPERM;
}

static int dev_release(struct inode *inode, struct file *file)
{
	struct misc_dev_data *dev_data = get_misc_dev_data(file);
    atomic_set(&dev_data->is_in_use, 0);

	return 0;
}

static int __init dev_init(void)
{
	int error;
	pr_info("Initializing...\n");

	init_misc_dev_group(&dev_group);

	error = misc_register(&key_dev);

	if (error) {
		printk(KERN_ERR
		       "An error occured during registering the key device! Error: %d",
		       error);
		return error;
	}

	error = misc_register(&encryption_dev);

	if (error) {
		printk(KERN_ERR
		       "An error occured during registering the encryption device! Error: %d",
		       error);
		return error;
	}

	return 0;
}

static void __exit dev_exit(void)
{
	misc_deregister(&key_dev);
	misc_deregister(&encryption_dev);

	pr_info("Exited...\n");
}

module_init(dev_init);
module_exit(dev_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Ege Bilecen");
MODULE_DESCRIPTION("Miscellaneous device written in C.");
