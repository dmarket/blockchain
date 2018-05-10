#include <stdio.h>
#include <string.h>

#include "dmbc_capi.h"

#ifndef DEBUG
#define DEBUG false
#endif

int main() {
    const char *public_key = "4e298e435018ab0a1430b6ebd0a0656be15493966d5ce86ed36416e24c411b9f";

    dmbc_error *err = dmbc_error_new();
    dmbc_builder *builder = dmbc_builder_create(0, 0, 2, 300, err);
    if (NULL == builder) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, "Error occured '%s'\n", msg);
        }
        goto free_error;
    }
    if (!dmbc_add_assets_set_public_key(builder, public_key, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, "Error occured %s\n", msg);
        }
        goto free_builder;
    } 
    if (!dmbc_add_assets_set_seed(builder, 102, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, "Error occured %s\n", msg);
        }
        goto free_builder;
    }
    dmbc_fees *fees = dmbc_fees_create(10, "0.1", 20, "0.2", 9, "0.999999", err);
    if (NULL == fees) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, "Error occured %s\n", msg);
        }
        goto free_builder;
    }
    if (!dmbc_add_assets_add_asset(builder, "Asset#10", 10, fees, public_key, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, "Error occured %s\n", msg);
        }
        goto free_fee;
    }
    if (!dmbc_add_assets_add_asset(builder, "Asset#00", 1000, fees, public_key, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, "Error occured %s\n", msg);
        }
        goto free_fee;
    }
    
    debug(builder);

free_fee: 
    dmbc_fees_free(fees);
free_builder:
    dmbc_builder_free(builder);
free_error:
    dmbc_error_free(err);
}