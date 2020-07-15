# Tests 

This directory contains all tests in mesatee-core-standalone.

## Run Tests

To run all tests with our build system:

```
# cd /teaclave/build
# make sgx-test
```

## Directory Structure

- `unit`:
  Unit tests are small and more focused, testing one module in isolation at a
  time, and can test private interfaces. This directory contains a test driver to
  test individual units/components or private interfaces. Test cases of unit
  tests are placed along with source code.

