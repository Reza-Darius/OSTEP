#include <err.h>
#include <fcntl.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#define BLOCK_SIZE (sizeof(int) + sizeof(char))

// this function assumes bytes is pointing at the first element of a block
void print_block(char *bytes) {
  // cast to int ptr then dereference
  // we know its little endian
  int count = *(int *) bytes;
  int letter = bytes[sizeof(int)];

  for (int i = 0; i < count; i++) {
    printf("%c", letter);
  }
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    printf("wunzip: file1 [file2 ...]\n");
    exit(EXIT_FAILURE);
  }

  size_t buf_len = BLOCK_SIZE << 10; // 5120 bytes
  char *buf = malloc(buf_len * sizeof(char));
  if (!buf) {
    err(EXIT_FAILURE, "malloc failed\n");
  }

  int fd;
  int nread = 0;
  int idx = 0;

  for (int i = 1; i < argc; i++) {
    fd = open(argv[i], O_RDONLY);
    if (fd < 0) {
      perror("open: ");
      err(EXIT_FAILURE, "failed to open file\n");
    }

    while ((nread = read(fd, buf, buf_len)) > 0) {
      if (nread % BLOCK_SIZE != 0) {
        err(EXIT_FAILURE, "invalid file size\n");
      }

      idx = 0;

      // read the buffer in 5 byte chunks
      while (idx < nread) {
        print_block(&buf[idx]);
        idx += BLOCK_SIZE;
      }
    }

    if (nread < 0) {
      perror("read: ");
      err(EXIT_FAILURE, "failed to read file\n");
    }
    close(fd);
  }

  free(buf);
  return EXIT_SUCCESS;
}
