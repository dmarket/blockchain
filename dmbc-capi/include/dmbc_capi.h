#ifndef _DMBC_CAPI_H
#define _DMBC_CAPI_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct dmbc_tx_add_asset dmbc_tx_add_asset;

typedef struct dmbc_builder dmbc_builder;

typedef struct dmbc_asset dmbc_asset;

typedef struct dmbc_fees dmbc_fees;

typedef struct dmbc_intermediary dmbc_intermediary;

typedef struct dmbc_exchange_offer dmbc_exchange_offer;

typedef struct dmbc_error dmbc_error;

#define TRANSFER_ID 200
#define ADD_ASSETS_ID 300
#define DELETE_ASSETS_ID 400
#define TRADE_ID 501
#define TRADE_INTERMEDIARY_ID 502
#define EXCHANGE_ID 601
#define EXCHANGE_INTERMEDIARY_ID 602

#define FEE_STRATEGY_RECIPIENT 1
#define FEE_STRATEGY_SENDER 2
#define FEE_STRATEGY_BOTH 3
#define FEE_STRATEGY_INTERMEDIARY 4

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

void dmbc_bytes_free(uint8_t *bytes, size_t length);

/*
    ADD ASSET
*/
dmbc_tx_add_asset *dmbc_tx_add_asset_create(
    const char *public_key,
    uint64_t seed,
    dmbc_error *error
);

void dmbc_tx_add_asset_free(dmbc_tx_add_asset *tx);

bool dmbc_tx_add_assets_add_asset(
    dmbc_tx_add_asset *tx,
    const char *name, 
    uint64_t amount,
    dmbc_fees *fees,
    const char *receiver_key,
    dmbc_error *error
);

uint8_t *dmbc_tx_add_assets_into_bytes(
    dmbc_tx_add_asset *tx, 
    size_t *length, 
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
    Transfer
*/
bool dmbc_transfer_set_from_public_key(
    dmbc_builder *builder,
    const char *public_key,
    dmbc_error *error
);

bool dmbc_transfer_set_to_public_key(
    dmbc_builder *builder,
    const char *public_key,
    dmbc_error *error
);

bool dmbc_transfer_set_seed(
    dmbc_builder *builder,
    uint64_t seed,
    dmbc_error *error
);

bool dmbc_transfer_set_amount(
    dmbc_builder *builder,
    uint64_t amount,
    dmbc_error *error
);

bool dmbc_transfer_add_asset(
    dmbc_builder *builder,
    dmbc_asset *asset,
    dmbc_error *error
);

/*
    Exchnage offer
*/
dmbc_exchange_offer *dmbc_exchange_offer_create(
    const char *sender_key,
    uint64_t sender_amount,
    const char *recipient_key,
    u_int8_t fee_strategy,
    dmbc_error *error
);

void dmbc_exchange_offer_free(dmbc_exchange_offer *offer);

bool dmbc_exchange_offer_recipient_add_asset(
    dmbc_exchange_offer *offer,
    dmbc_asset *asset,
    dmbc_error *error
);

bool dmbc_exchange_offer_sender_add_asset(
    dmbc_exchange_offer *offer,
    dmbc_asset *asset,
    dmbc_error *error
);

uint8_t* dmbc_exchange_offer_into_bytes(
    dmbc_exchange_offer *offer,
    size_t *length,
    dmbc_error *error
);

void dmbc_exchange_offer_bytes_free(uint8_t *tx_ptr, size_t length);

/*
    Asset
*/
dmbc_asset *dmbc_asset_create(
    const char *id,
    uint64_t amount,
    dmbc_error *error
);

void dmbc_asset_free(dmbc_asset *asset);

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
    Intemediary
*/
dmbc_intermediary *dmbc_intermediary_create(
    const char *public_key,
    uint64_t commission
);

void dmbc_intermediary_free(dmbc_intermediary *intermediary);

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