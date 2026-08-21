#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef int32_t i32;

i32 compress(FILE **files, i32 n_files, FILE *out, char *buf, size_t buf_len);
void print_count(i32 count, char letter, FILE* out);
