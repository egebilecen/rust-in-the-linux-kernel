#include "main.h"
#include "util.h"
#include "present80.h"

/* Forward function declarations. */
static int dev_open(struct inode *, struct file *);
static ssize_t dev_read(struct file *, char __user *, size_t, loff_t *);
static ssize_t dev_write(struct file *, const char __user *, size_t, loff_t *);
static int dev_release(struct inode *, struct file *);

/* File operations for this device. */
static struct file_operations dev_fops = { .open = dev_open,
					   .read = dev_read,
					   .write = dev_write,
					   .release = dev_release };

/* Key device setup. */
static struct miscdevice key_dev = { .minor = MISC_DYNAMIC_MINOR,
				     .name = DEVICE_NAME_KEY,
				     .fops = &dev_fops };

/* Encryption device setup. */
static struct miscdevice encryption_dev = { .minor = MISC_DYNAMIC_MINOR,
					    .name = DEVICE_NAME_ENCRYPTION,
					    .fops = &dev_fops };

/* Empty device group. Initialized in "dev_init()". */
static struct misc_dev_group dev_group;

/* Gets the "misc_dev_data" from "file" ptr. */
static struct misc_dev_data *get_misc_dev_data(struct file *file)
{
	int minor = iminor(file->f_inode);

	return minor == key_dev.minor	     ? &dev_group.key :
	       minor == encryption_dev.minor ? &dev_group.encryption :
					       NULL;
}

/* Initialize the device with default values. */
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

/* Initialize the device group. */
static void init_misc_dev_group(struct misc_dev_group *group)
{
	init_misc_dev_data(&group->key, KEY_DEVICE);
	init_misc_dev_data(&group->encryption, ENCRYPTION_DEVICE);
}

/* Called when device is opened. */
static int dev_open(struct inode *inode, struct file *file)
{
	int return_val = 0;

	/* Get the related device data. */
	struct misc_dev_data *dev_data = get_misc_dev_data(file);

	/* Acquire lock. */
	mutex_lock(&dev_data->lock);

	/* Check if device is in use already. Return "EBUSY" error if in use. */
	if (dev_data->is_in_use) {
		return_val = -EBUSY;
		goto out;
	}

	/* Set the device as in use. */
	dev_data->is_in_use = true;

	/* Clear the input and output buffer of the device. */
	buffer_zeros(dev_data->in_buffer, MAX_BUFFER_SIZE);
	buffer_zeros(dev_data->out_buffer, MAX_BUFFER_SIZE);

out:
	/* Release the lock. */
	mutex_unlock(&dev_data->lock);
	return return_val;
}

/* Called when device is read. */
static ssize_t dev_read(struct file *file, char __user *buff, size_t len,
			loff_t *offset)
{
	int return_val = 0;

	/* Get the related device data. */
	struct misc_dev_data *dev_data = get_misc_dev_data(file);
	struct misc_dev_data *key_dev_data = NULL;

	/* Acquire the lock. */
	mutex_lock(&dev_data->lock);

	/* Prevent read operation for the key device as it doesn't support */
	/* such operation. */
	if (dev_data->type == KEY_DEVICE) {
		pr_err("Key device doesn't support read operation.\n");

		return_val = -EPERM;
		goto out;
	}

	/* Partial read is not supported. Return "EINVAL" error. */
	if (*offset != 0) {
		pr_err("Encryption device doesn't support partial read. Offset is not 0.\n");

		return_val = -EINVAL;
		goto out;
	}

	/* Encryption device needs the key set in the key device. Get the key device data. */
	key_dev_data = &dev_group.key;
	/* Acquire its lock. */
	mutex_lock(&key_dev_data->lock);

	/* Perform the PRESENT-80 encryption on plaintext bytes using key bytes. */
	present80_encrypt(key_dev_data->in_buffer, dev_data->in_buffer,
			  dev_data->out_buffer);

	/* Copy the encryption result to the user space. */
	return_val = simple_read_from_buffer(
		buff, len, offset, dev_data->out_buffer, PRESENT80_BLOCK_SIZE);

out:
	/* Release the lock(s). */
	if (key_dev_data)
		mutex_unlock(&key_dev_data->lock);
	mutex_unlock(&dev_data->lock);

	return return_val;
}

/* Called when some bytes written into device. */
static ssize_t dev_write(struct file *file, const char __user *buff, size_t len,
			 loff_t *offset)
{
	ssize_t ret_val = 0;
	struct misc_dev_data *dev_data = NULL;

	/* Partial write is not supported. Return "EINVAL" error. */
	if (*offset != 0) {
		pr_err("PRESENT80 devices doesn't support partial write. Offset is not 0.\n");

		ret_val = -EINVAL;
		goto out;
	}

	/* Get the related device data. */
	dev_data = get_misc_dev_data(file);
	/* Acquire the lock. */
	mutex_lock(&dev_data->lock);

	/* Validate the size of the incoming data before proceeding further. */
	switch (dev_data->type) {
	case KEY_DEVICE:
		if (len != PRESENT80_KEY_SIZE) {
			pr_err("Key device requires %d bytes to be written. Found %d bytes.\n",
			       PRESENT80_KEY_SIZE, len);

			ret_val = -EINVAL;
			goto out;
		}
		break;

	case ENCRYPTION_DEVICE:
		if (len != PRESENT80_BLOCK_SIZE) {
			pr_err("Encryption device requires %d bytes to be written. Found %d bytes.\n",
			       PRESENT80_BLOCK_SIZE, len);

			ret_val = -EINVAL;
			goto out;
		}
		break;
	}

	/* Copy user space data to the device's input buffer. */
	ret_val = simple_write_to_buffer(dev_data->in_buffer, MAX_BUFFER_SIZE,
					 offset, buff, len);

out:
	/* Release the lock. */
	mutex_unlock(&dev_data->lock);
	return ret_val;
}

/* Called when device is closed. */
static int dev_release(struct inode *inode, struct file *file)
{
	/* Get the related device data. */
	struct misc_dev_data *dev_data = get_misc_dev_data(file);
	/* Acquire the lock. */
	mutex_lock(&dev_data->lock);

	/* Set the device as not in use. */
	dev_data->is_in_use = false;

	/* Release the lock. */
	mutex_unlock(&dev_data->lock);
	return 0;
}

/* Called when the module is loaded. */
static int __init dev_init(void)
{
	int error;

	/* Initialize device group. */
	init_misc_dev_group(&dev_group);

	/* Try to register the key device. */
	error = misc_register(&key_dev);

	if (error) {
		printk(KERN_ERR
		       "An error occured during registering the key device. Error: %d\n",
		       error);
		return error;
	}

	/* Try to register the encryption device. */
	error = misc_register(&encryption_dev);

	if (error) {
		printk(KERN_ERR
		       "An error occured during registering the encryption device. Error: %d\n",
		       error);
		return error;
	}

	return 0;
}

/* Called when the module is unloaded. */
static void __exit dev_exit(void)
{
	/* De-register the all devices. */
	misc_deregister(&key_dev);
	misc_deregister(&encryption_dev);
}

/* Export "init" and "exit" module functions. */
module_init(dev_init);
module_exit(dev_exit);

/* Set module properties. */
MODULE_LICENSE("GPL");
MODULE_AUTHOR("Ege Bilecen");
MODULE_DESCRIPTION("Miscellaneous device written in C.");
