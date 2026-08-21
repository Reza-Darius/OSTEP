#include <stdio.h>
#include <stdlib.h>

#define BUF_SIZE 2024

int main(int argc, char *argv[]) {
  if (argc < 2) {
    exit(EXIT_SUCCESS);
  }

  char *line = malloc(BUF_SIZE * sizeof(char));
  if (!line) {
    printf("malloc failed\n");
    exit(EXIT_FAILURE);
  }

  for (int i = 1; i < argc; i++) {
    FILE *file = fopen(argv[i], "r");

    if (!file) {
      printf("wcat: cannot open file\n");
      exit(EXIT_FAILURE);
    }

    while (fgets(line, BUF_SIZE, file)) {
      fprintf(stdout, "%s", line);
    }

    fclose(file);
  }

  free(line);
  return EXIT_SUCCESS;
}
