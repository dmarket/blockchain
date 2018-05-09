#include <stdio.h>
#include <string.h>

#include "dmbc_capi.h"

#ifndef DEBUG
#define DEBUG false
#endif

int main() {
    const char *public_key = "4e298e435018ab0a1430b6ebd0a0656be15493966d5ce86ed36416e24c411b9f";
    const char *secret_key = "a5bc1d2f4de5f48d95e6b607f9975a2021a176e8b9bf74735c92653318ad98d34e298e435018ab0a1430b6ebd0a0656be15493966d5ce86ed36416e24c411b9f";

    dmbc_error *err = dmbc_error_new();
    dmbc_builder *builder = dmbc_builder_create(public_key, secret_key, 0, 0, 300, err);
    if (NULL == builder) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, "Error occured '%s'\n", msg);
        }
    }
    if (NULL != builder) {
        dmbc_builder_free(builder);
    }
    dmbc_error_free(err);
}