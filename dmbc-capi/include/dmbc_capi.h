#ifndef _DMBC_CAPI_H
#define _DMBC_CAPI_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct dmbc_builder dmbc_builder;

typedef struct dmbc_asset dmbc_asset;

typedef struct dmbc_fees dmbc_fees;

typedef struct dmbc_error dmbc_error;

/*
    BUILDER
*/
dmbc_builder *dmbc_builder_create(
    uint8_t network_id,
    uint8_t protocol_version,
    uint16_t service_id,
    uint16_t message_type,
    dmbc_error *error
);

void dmbc_builder_free(dmbc_builder *builder);

uint8_t *dmbc_builder_tx_create(
    dmbc_builder *builder,
    size_t *length,
    dmbc_error *error
);

void dmbc_builder_tx_free(uint8_t *tx_ptr, size_t length);

/*
    ADD ASSET
*/
bool dmbc_add_assets_set_public_key(
    dmbc_builder *builder, 
    const char *public_key, 
    dmbc_error *error
);

bool dmbc_add_assets_set_seed(
    dmbc_builder *builder, 
    uint64_t seed, 
    dmbc_error *error
);

bool dmbc_add_assets_add_asset(
    dmbc_builder *builder,
    const char *name, 
    uint64_t amount,
    dmbc_fees *fees,
    const char *receiver_key,
    dmbc_error *error
);

/*
    Delete Assets
*/
bool dmbc_delete_assets_set_public_key(
    dmbc_builder *builder,
    const char *public_key,
    dmbc_error *error
);

bool dmbc_delete_assets_set_seed(
    dmbc_builder *builder,
    uint64_t seed,
    dmbc_error *error
);

bool dmbc_delete_assets_add_asset(
    dmbc_builder *builder,
    dmbc_asset *asset,
    dmbc_error *error
);

/*
    Asset
*/
dmbc_asset *dmbc_asset_create(
    const char *id,
    uint64_t amount
);

void dmbc_asset_fee(dmbc_asset *asset);

/*
    FEES
*/
dmbc_fees *dmbc_fees_create(
    uint64_t trade_fixed, 
    const char *trade_fraction,
    uint64_t exchange_fixed, 
    const char *exchange_fraction,
    uint64_t transfer_fixed, 
    const char *transfer_fraction,
    dmbc_error *error
);

void dmbc_fees_free(dmbc_fees *fees);

/*
    ERROR
*/
dmbc_error *dmbc_error_new();

const char *dmbc_error_message(dmbc_error *error);

void dmbc_error_free(dmbc_error *error);

/*
    DEBUG
*/
void debug(dmbc_builder *builder);

#ifdef __cplusplus
}
#endif

#endif