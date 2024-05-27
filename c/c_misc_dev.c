#include "linux/mutex.h"
#include "linux/miscdevice.h"
#include "linux/fs.h"

#define pr_fmt(fmt) "%s: " fmt, KBUILD_MODNAME

#define DEVICE_PREFIX "present80"
#define DEVICE_NAME_KEY DEVICE_PREFIX "_key"
#define DEVICE_NAME_ENCRYPTION DEVICE_PREFIX "_encrypt"

/* In bytes. */
#define BUFFER_SIZE 10

#define buffer_zeroes(buff, size) memset(buff, 0, size)
struct misc_dev_data {
    struct mutex lock;
    bool is_in_use;
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

static void init_misc_dev_data(struct misc_dev_data *data)
{
    mutex_init(&data->lock);
    data->is_in_use = false;

    /* When the device is opened for the first time,
     * buffer is set to all zeroes so we don't need
     * to do anything about it.
     */
}

static void init_misc_dev_group(struct misc_dev_group *group)
{
    init_misc_dev_data(&group->key);
    init_misc_dev_data(&group->encryption);
}

static int dev_open(struct inode *inode, struct file *file)
{
    int return_code = 0;
	struct misc_dev_data *dev_data = get_misc_dev_data(file);

    mutex_lock(&dev_data->lock);

    if(dev_data->is_in_use) {
        return_code = -EBUSY;
        goto out;
    }

    dev_data->is_in_use = true;
    memset(dev_data->in_buffer, 0, sizeof(dev_data->in_buffer));
    memset(dev_data->out_buffer, 0, sizeof(dev_data->out_buffer));
	buffer_zeroes(dev_data->in_buffer, BUFFER_SIZE);
	buffer_zeroes(dev_data->out_buffer, BUFFER_SIZE);

out:
    mutex_unlock(&dev_data->lock);
	return return_code;
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
    mutex_lock(&dev_data->lock);

    dev_data->is_in_use = false;

    mutex_unlock(&dev_data->lock);
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
