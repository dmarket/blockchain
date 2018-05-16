#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#include "dmbc_capi.h"

#ifndef DEBUG
#define DEBUG false
#endif

const char *error_msg = "Error occured '%s'\n";
const char *output = "\nTransaction length is %lu hex %s\n";

void print_hex(const uint8_t *hex, size_t length) {
    for (int i = 0; i < length; ++i) {
        fprintf(stdout, "%02x", hex[i]);
    }
    puts("");
}

void add_assets() {
    const char *public_key = "4e298e435018ab0a1430b6ebd0a0656be15493966d5ce86ed36416e24c411b9f";

    dmbc_error *err = dmbc_error_new();
    dmbc_builder *builder = dmbc_builder_create(0, 0, 2, ADD_ASSETS_ID, err);
    if (NULL == builder) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_error;
    }
    if (!dmbc_add_assets_set_public_key(builder, public_key, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_builder;
    } 
    if (!dmbc_add_assets_set_seed(builder, 102, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_builder;
    }
    dmbc_fees *fees = dmbc_fees_create(10, "0.1", 20, "0.2", 9, "0.999999", err);
    if (NULL == fees) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_builder;
    }
    if (!dmbc_add_assets_add_asset(builder, "Asset#10", 10, fees, public_key, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_fee;
    }
    if (!dmbc_add_assets_add_asset(builder, "Asset#00", 1000, fees, public_key, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_fee;
    }
    
    size_t length = 0;
    uint8_t *buffer = dmbc_builder_tx_create(builder, &length, err);
    if (NULL == buffer) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_fee;
    }

    print_hex(buffer, length);

    dmbc_builder_tx_free(buffer, length);
free_fee: 
    dmbc_fees_free(fees);
free_builder:
    dmbc_builder_free(builder);
free_error:
    dmbc_error_free(err);
}

void delete_assets() {
    const char *public_key = "4e298e435018ab0a1430b6ebd0a0656be15493966d5ce86ed36416e24c411b9f";

    dmbc_error *err = dmbc_error_new();
    dmbc_builder *builder = dmbc_builder_create(0, 0, 2, DELETE_ASSETS_ID, err);
    if (NULL == builder) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_error;
    }
    if (!dmbc_delete_assets_set_public_key(builder, public_key, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_builder;
    } 
    if (!dmbc_delete_assets_set_seed(builder, 102, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_builder;
    }
    dmbc_asset *asset = dmbc_asset_create("00001111222233334444555566667777", 10, err);
    if (NULL == asset) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_builder;
    }
    if (!dmbc_delete_assets_add_asset(builder, asset, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_asset;
    }
    

    size_t length = 0;
    uint8_t *buffer = dmbc_builder_tx_create(builder, &length, err);
    if (NULL == buffer) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_asset;
    }

    print_hex(buffer, length);

    dmbc_builder_tx_free(buffer, length);
free_asset: 
    dmbc_asset_free(asset);
free_builder:
    dmbc_builder_free(builder);
free_error:
    dmbc_error_free(err);
}

void transfer() {
    const char *from_public_key = "4e298e435018ab0a1430b6ebd0a0656be15493966d5ce86ed36416e24c411b9f";
    const char *to_public_key = "00098e435018ab0a1430b6ebd0a0656be15493966d5ce86ed36416e24c411000";

    dmbc_error *err = dmbc_error_new();
    dmbc_builder *builder = dmbc_builder_create(0, 0, 2, TRANSFER_ID, err);
    if (NULL == builder) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_error;
    }
    if (!dmbc_transfer_set_from_public_key(builder, from_public_key, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_builder;
    } 
    if (!dmbc_transfer_set_to_public_key(builder, to_public_key, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_builder;
    } 
    if (!dmbc_transfer_set_seed(builder, 102, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_builder;
    }
    if (!dmbc_transfer_set_amount(builder, 10000000, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_builder;
    }
    dmbc_asset *asset = dmbc_asset_create("00001111222233334444555566667777", 10, err);
    if (NULL == asset) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_builder;
    }
    if (!dmbc_transfer_add_asset(builder, asset, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_asset;
    }
    
    size_t length = 0;
    uint8_t *buffer = dmbc_builder_tx_create(builder, &length, err);
    if (NULL == buffer) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_asset;
    }

    print_hex(buffer, length);

    dmbc_builder_tx_free(buffer, length);
free_asset: 
    dmbc_asset_free(asset);
free_builder:
    dmbc_builder_free(builder);
free_error:
    dmbc_error_free(err);
}

void exchange_offer() {
    const char *sender_public_key = "4e298e435018ab0a1430b6ebd0a0656be15493966d5ce86ed36416e24c411b9f";
    const char *recipient_public_key = "00098e435018ab0a1430b6ebd0a0656be15493966d5ce86ed36416e24c411000";

    dmbc_error *err = dmbc_error_new();
    dmbc_exchange_offer *offer = dmbc_exchange_offer_create(sender_public_key, 10000, recipient_public_key, 1, err);
    if (NULL == offer) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_error;
    }
    dmbc_asset *asset = dmbc_asset_create("00001111222233334444555566667777", 10, err);
    if (NULL == asset) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_offer;
    }

    if (!dmbc_exchange_offer_recipient_add_asset(offer, asset, err)) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_asset;
    }

    size_t length = 0;
    uint8_t *buffer = dmbc_exchange_offer_into_bytes(offer, &length, err);
    if (NULL == buffer) {
        const char *msg = dmbc_error_message(err);
        if (NULL != msg) {
            fprintf(stderr, error_msg, msg);
        }
        goto free_asset;
    }

    print_hex(buffer, length);

    dmbc_exchange_offer_bytes_free(buffer, length);

free_asset:
    dmbc_asset_free(asset);
free_offer:
    dmbc_exchange_offer_free(offer);
free_error:
    dmbc_error_free(err);
}

int main() {
#if 0
    add_assets();
    delete_assets();
    transfer();
#endif
    exchange_offer();
}