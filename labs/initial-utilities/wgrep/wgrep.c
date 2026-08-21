#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void search_needle(char *needle, int len_needle, char *line, int len_line) {
  int i, j;

  for (i = 0; i + len_needle < len_line; i++) {
    for (j = 0; j < len_needle; j++) {
      if (needle[j] != line[j + i]) {
        break;
      }
    }

    if (j == len_needle) {
      fwrite(line, len_line, 1, stdout);
      return;
    }
  }

  return;
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    printf("wgrep: searchterm [file ...]\n");
    exit(EXIT_FAILURE);
  }

  char *needle = argv[1];
  int len_needle = strlen(needle);

  if (len_needle == 0) {
    exit(EXIT_SUCCESS);
  }

  FILE *file = stdin;
  char *line = NULL;
  int len_line;
  size_t size = 0;

  if (argc == 2) {
    while ((len_line = getline(&line, &size, file)) != -1) {
      search_needle(needle, len_needle, line, len_line);
    };
  }

  for (int i = 2; i < argc; i++) {
    file = fopen(argv[i], "r");
    if (!file) {
      printf("wgrep: cannot open file\n");
      exit(EXIT_FAILURE);
    }

    while ((len_line = getline(&line, &size, file)) != -1) {
      search_needle(needle, len_needle, line, len_line);
    };

    fclose(file);
  }

  free(line);
  return EXIT_SUCCESS;
}
