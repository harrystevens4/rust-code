
# Usage

## Example

Say I wanted to embed an American dictionary into my program, how would I do that?

1. Generate the object file with the dictionary embedded: `elf-embed-file /usr/share/dict/american-english -o dictionary.o`
2. Create a sample program to use it: 
```
//====== example.c ======
#include <stdio.h>
#include <stdint.h>                                                                           
//see "symbol name conversion" for more info
extern char _usr_share_dict_american_english[];
extern uint64_t _usr_share_dict_american_english_size;

int main(){
    char *data = _usr_share_dict_american_english;
    uint64_t data_length = _usr_share_dict_american_english_size;
    printf("%.*s\n",(int)data_length,data);
    return 0;
}
```
3. compile and link: `gcc -o example example.c dictionary.o`

## How it works

It creates an object file with a symbol in the `.rodata` section for the file and file size of each of the files provided on the command line. This can then be linked with a program and the symbols used directly using the `extern` keyword.
The `<...>_size` symbol will always be a `uint64_t` and the file data symbol will always be the length of the size symbol provided. No null terminator is added. The endianness of the `<file>_size` symbol is whatever the host's endianness is.

## Symbol name conversion

`-`, ` `, `/` and `.` are all converted into `_`, and any other non alphanumeric character is simply removed.
For example, `/usr/share/dict/words.txt` would become `_usr_share_dict_words_txt` and `i love apples!!!!` would become `i_love_apples`
The size symbol will always be the data symbol + `_size`, for example `lets_go_` would become `lets_go__size` or `hi` would become `hi_size`
