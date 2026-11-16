#include <bits/time.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <time.h>
#include <unistd.h>

// number pages
// iterations

int main(int argc, char *argv[]) {
  if (argc != 3)
    return -1;

  long n_pages = atol(argv[1]), n_iterations = atol(argv[2]);
  int pagesize = getpagesize();
  size_t size = n_pages * pagesize;

  printf("Running %ld pages at %ld iterations\n", n_pages, n_iterations);

  unsigned char *addr = mmap(NULL, size, PROT_READ | PROT_WRITE,
                             MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);

  struct timespec now, elapsed;
  if (clock_gettime(CLOCK_MONOTONIC, &now) == -1) {
    perror("failed to get now time\n");
    return EXIT_FAILURE;
  };

  for (int i = 0; i < n_iterations; i++)
    for (int p = 0; p < n_pages; p++)
      addr[p * pagesize] += 1;

  if (clock_gettime(CLOCK_MONOTONIC, &elapsed) == -1) {
    perror("failed to get elapsed time\n");
    return EXIT_FAILURE;
  };

  double elapsed_time =
      (elapsed.tv_sec - now.tv_sec) * 1e9 + (elapsed.tv_nsec - now.tv_nsec);
  double n_access = n_pages * n_iterations;
  double average = elapsed_time / n_access;

  printf("average access time %f ns\n", average);
  munmap(addr, size);

  return EXIT_SUCCESS;
}
