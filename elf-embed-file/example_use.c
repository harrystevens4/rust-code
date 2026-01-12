#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

int main(){
	void *pointer = dlsym(dlopen(NULL,RTLD_LAZY),"file_content");
	if (pointer == NULL){
		fprintf(stderr,"%s\n",dlerror());
		return 1;
	}
	uint64_t data_length = *(uint64_t *)pointer;
	char *data = pointer+sizeof(uint64_t);
	printf("%*.s\n",(int)data_length,data);
	return 0;
}
