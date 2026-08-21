#include "wzip.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <err.h>

#define BUF_SIZE 1024

void print_count(i32 count, char letter, FILE *out) {
  fwrite(&count, sizeof(i32), 1, out);
  fwrite(&letter, sizeof(char), 1, out);
  return;
}

// compresses bytes from input files and writes them to output stream
i32 compress(FILE **files, i32 n_files, FILE *out, char *buf, size_t buf_len) {
  char comp_char;
  bool first = true;
  i32 nread = 0;
  i32 idx = 0;
  i32 count = 0;
  FILE *cur_file;

  // iterate through files
  for (int f_idx = 0; f_idx < n_files; f_idx++) {
    cur_file = files[f_idx];

    nread = fread(buf, sizeof(char), buf_len, cur_file);
    if (ferror(cur_file)) {
      printf("empty file or error occured\n");
      return -1;
    }
    idx = 0;

    // one time init
    if (first) {
      comp_char = buf[idx];
      count = 1;
      idx++;
      first = false;
    }

    // iterate through file
    while (nread > 0) {
      // iterate through the buffer
      while (idx < nread) {
        if (buf[idx] == comp_char) {
          if (count == INT32_MAX) {
            printf("count overflow\n");
            return -1;
          };
          count++;
        } else {
          print_count(count, comp_char, out);
          comp_char = buf[idx];
          count = 1;
        }
        idx++;
      }

      nread = fread(buf, sizeof(char), buf_len, cur_file);
      if (ferror(cur_file)) {
        printf("coudlnt read from the file\n");
        return -1;
      }
      idx = 0;
    }
  }

  if (!first) {
    print_count(count, comp_char, out);
  }

  fflush(out);
  return 0;
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    printf("wzip: file1 [file2 ...]\n");
    exit(EXIT_FAILURE);
  }

  i32 res, nfiles, buf_len;
  nfiles = argc - 1;
  char *buf;
  FILE *file;
  FILE *files[nfiles];

  buf_len = BUF_SIZE * sizeof(char);
  buf = malloc(buf_len);
  if (!buf) {
    printf("malloc error\n");
    return -1;
  }

  // open files
  for (int i = 1; i < argc; i++) {
    file = fopen(argv[i], "r");
    if (!file) {
      printf("wgrep: cannot open file\n");
      exit(EXIT_FAILURE);
    }

    files[i - 1] = file;
  }

  if ((res = compress(files, nfiles, stdout, buf, buf_len) == -1)) {
    printf("compression error\n");
    exit(EXIT_FAILURE);
  };

  // close files
  for (int i = 0; i < argc - 1; i++) {
    fclose(files[i]);
  }

  free(buf);
  return EXIT_SUCCESS;
}
