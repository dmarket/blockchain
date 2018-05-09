#include <stdio.h>
#include <string.h>

#include "dmbc_capi.h"

#ifndef DEBUG
#define DEBUG false
#endif

int main() {
    dmbc_error *err = dmbc_error_new();
    dmbc_builder *builder = dmbc_builder_create("hello", "hello", 0, 0, 300, err);
    const char *msg = dmbc_error_message(err);
    if (NULL != msg) {
        fprintf(stderr, "Error occured '%s'", msg);
    }
    dmbc_builder_free(builder);
    dmbc_error_free(err);
}