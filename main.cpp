#include "rust.hpp"

int main() {
  shared::Dummy data = {.a = 7};
  dummy::print_dummy(data);
  return 0;
}
