#include <stdio.h>
#include <stdint.h>

extern char _usr_share_dict_american_english[];
extern uint64_t _usr_share_dict_american_english_size;

int main(){
	char *data = _usr_share_dict_american_english;
	uint64_t data_length = _usr_share_dict_american_english_size;
	printf("%.*s\n",(int)data_length,data);
	return 0;
}
