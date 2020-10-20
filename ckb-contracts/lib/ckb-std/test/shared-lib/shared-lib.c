typedef unsigned long size_t;

__attribute__((visibility("default"))) int
plus_42(size_t num) {
  return 42 + num;
}

__attribute__((visibility("default"))) char *
foo() {
  return "foo";
}
