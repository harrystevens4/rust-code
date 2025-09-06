#include <sys/ioctl.h>
#include <stdint.h>
#include <limits.h>
#include <unistd.h>
int32_t get_term_width(){
	struct winsize size;
	int result = ioctl(STDOUT_FILENO,TIOCGWINSZ,&size);
	if (result < 0) return INT_MAX;
	return size.ws_col;
}
