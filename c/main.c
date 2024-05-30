#include "main.h"
#include "util.h"
#include "present80.h"

static int dev_open(struct inode *, struct file *);
static ssize_t dev_read(struct file *, char __user *, size_t, loff_t *);
static ssize_t dev_write(struct file *, const char __user *, size_t, loff_t *);
static int dev_release(struct inode *, struct file *);

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

static struct misc_dev_data *get_misc_dev_data(struct file *file)
{
	int minor = iminor(file->f_inode);

	return minor == key_dev.minor	     ? &dev_group.key :
	       minor == encryption_dev.minor ? &dev_group.encryption :
					       NULL;
}

static void init_misc_dev_data(struct misc_dev_data *data,
			       enum misc_dev_type type)
{
	mutex_init(&data->lock);

	data->type = type;
	data->is_in_use = false;

	/* When the device is opened for the first time,
     * buffer is set to all zeroes so we don't need
     * to do anything about it.
     */
}

static void init_misc_dev_group(struct misc_dev_group *group)
{
	init_misc_dev_data(&group->key, KEY_DEVICE);
	init_misc_dev_data(&group->encryption, ENCRYPTION_DEVICE);
}

static int dev_open(struct inode *inode, struct file *file)
{
	int return_val = 0;
	struct misc_dev_data *dev_data = get_misc_dev_data(file);

	mutex_lock(&dev_data->lock);

	if (dev_data->is_in_use) {
		return_val = -EBUSY;
		goto out;
	}

	dev_data->is_in_use = true;
	buffer_zeros(dev_data->in_buffer, MAX_BUFFER_SIZE);
	buffer_zeros(dev_data->out_buffer, MAX_BUFFER_SIZE);

out:
	mutex_unlock(&dev_data->lock);
	return return_val;
}

static ssize_t dev_read(struct file *file, char __user *buff, size_t len,
			loff_t *offset)
{
	int return_val = 0;

	struct misc_dev_data *dev_data = get_misc_dev_data(file);
	struct misc_dev_data *key_dev_data = NULL;

	mutex_lock(&dev_data->lock);

	if (dev_data->type == KEY_DEVICE) {
		pr_err("Key device doesn't support read operation.\n");

		return_val = -EPERM;
		goto out;
	}

	if (*offset != 0) {
		pr_err("Encryption device doesn't support partial read. Offset is not 0.\n");

		return_val = -EINVAL;
		goto out;
	}

	key_dev_data = &dev_group.key;
	mutex_lock(&key_dev_data->lock);

	present80_encrypt(key_dev_data->in_buffer, dev_data->in_buffer,
			  dev_data->out_buffer);

	return_val = simple_read_from_buffer(buff, len, offset,
					     dev_data->out_buffer,
					     ENCRYPTION_BUFFER_SIZE);

out:
	if (key_dev_data)
		mutex_unlock(&key_dev_data->lock);
	mutex_unlock(&dev_data->lock);

	return return_val;
}

static ssize_t dev_write(struct file *file, const char __user *buff, size_t len,
			 loff_t *offset)
{
	ssize_t ret_val = 0;
	struct misc_dev_data *dev_data = NULL;

	if (*offset != 0) {
		pr_err("PRESENT80 devices doesn't support partial write. Offset is not 0.\n");

		ret_val = -EINVAL;
		goto out;
	}

	dev_data = get_misc_dev_data(file);
	mutex_lock(&dev_data->lock);

	switch (dev_data->type) {
	case KEY_DEVICE:
		if (len != KEY_BUFFER_SIZE) {
			pr_err("Key device requires %d bytes to be written. Found %d bytes.\n",
			       KEY_BUFFER_SIZE, len);

			ret_val = -EINVAL;
			goto out;
		}
		break;

	case ENCRYPTION_DEVICE:
		if (len != ENCRYPTION_BUFFER_SIZE) {
			pr_err("Encryption device requires %d bytes to be written. Found %d bytes.\n",
			       ENCRYPTION_BUFFER_SIZE, len);

			ret_val = -EINVAL;
			goto out;
		}
		break;
	}

	ret_val = simple_write_to_buffer(dev_data->in_buffer, MAX_BUFFER_SIZE,
					 offset, buff, len);

out:
	mutex_unlock(&dev_data->lock);
	return ret_val;
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
		       "An error occured during registering the key device. Error: %d\n",
		       error);
		return error;
	}

	error = misc_register(&encryption_dev);

	if (error) {
		printk(KERN_ERR
		       "An error occured during registering the encryption device. Error: %d\n",
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
