#include <stdio.h>
#include <string.h>

#include "dmbc_capi.h"

#ifndef DEBUG
#define DEBUG false
#endif

int bytes_to_hex(const uint8_t *in, size_t insz,
                 char *out, size_t outsz) {
    if (outsz % 2 != 0 || outsz / 2 != insz) {
        return -1;
    }
    
    const uint8_t * pin = in;
    const char * hex = "0123456789abcdef";
    char * pout = out;
    for(; pin < in+insz; pout +=2, pin++){
        pout[0] = hex[(*pin>>4) & 0xF];
        pout[1] = hex[ *pin     & 0xF];
        if (pout + 2 - out > (int)outsz){
            /* Better to truncate output string than overflow buffer */
            /* it would be still better to either return a status */
            /* or ensure the target buffer is large enough and it never happen */
            break;
        }
    }
    pout[-1] = 0;
    return 0;
}


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
    
    // debug(builder);
    size_t length = 0;
    uint8_t *buffer = dmbc_builder_tx_create(builder, &length, err);
    if (NULL == buffer) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, "Error occured %s\n", msg);
        }
        goto free_fee;
    }

    char hex[346*2 + 1] = { 0 };
    bytes_to_hex(buffer, length, hex, sizeof(hex) / sizeof(char) - 1);

    fprintf(stdout, "\nlength is %lu\n", length);
    fprintf(stdout, "\n the HEX \n%s\n", hex);

free_tx:
    dmbc_builder_tx_free(buffer, length);
free_fee: 
    dmbc_fees_free(fees);
free_builder:
    dmbc_builder_free(builder);
free_error:
    dmbc_error_free(err);
}