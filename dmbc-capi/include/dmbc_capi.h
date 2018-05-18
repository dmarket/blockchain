#ifndef _DMBC_CAPI_H
#define _DMBC_CAPI_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct dmbc_tx_transfer dmbc_tx_transfer;

typedef struct dmbc_tx_add_assets dmbc_tx_add_assets;

typedef struct dmbc_tx_delete_assets dmbc_tx_delete_assets;

typedef struct dmbc_asset dmbc_asset;

typedef struct dmbc_trade_asset dmbc_trade_asset;

typedef struct dmbc_fees dmbc_fees;

typedef struct dmbc_intermediary dmbc_intermediary;

typedef struct dmbc_exchange_offer dmbc_exchange_offer;

typedef struct dmbc_tx_exchange dmbc_tx_exchange;

typedef struct dmbc_exchange_offer_intermediary dmbc_exchange_offer_intermediary;

typedef struct dmbc_tx_exchange_intermediary dmbc_tx_exchange_intermediary;

typedef struct dmbc_trade_offer dmbc_trade_offer;

typedef struct dmbc_tx_trade dmbc_tx_trade;

typedef struct dmbc_error dmbc_error;

#define FEE_STRATEGY_RECIPIENT 1
#define FEE_STRATEGY_SENDER 2
#define FEE_STRATEGY_BOTH 3
#define FEE_STRATEGY_INTERMEDIARY 4

void dmbc_bytes_free(uint8_t *bytes, size_t length);

/*
    ADD ASSET
*/
dmbc_tx_add_assets *dmbc_tx_add_assets_create(
    const char *public_key,
    uint64_t seed,
    dmbc_error *error
);

void dmbc_tx_add_asset_free(dmbc_tx_add_assets *tx);

bool dmbc_tx_add_assets_add_asset(
    dmbc_tx_add_assets *tx,
    const char *name, 
    uint64_t amount,
    dmbc_fees *fees,
    const char *receiver_key,
    dmbc_error *error
);

uint8_t *dmbc_tx_add_assets_into_bytes(
    dmbc_tx_add_assets *tx, 
    size_t *length, 
    dmbc_error *error
);

/*
    Delete Assets
*/
dmbc_tx_delete_assets *dmbc_tx_delete_assets_create(
    const char *public_key,
    uint64_t seed,
    dmbc_error *error
);

void dmbc_tx_delete_assets_free(dmbc_tx_delete_assets *tx);

bool dmbc_tx_delete_assets_add_asset(
    dmbc_tx_delete_assets *tx,
    dmbc_asset *asset,
    dmbc_error *error
);

uint8_t *dmbc_tx_delete_assets_into_bytes(
    dmbc_tx_delete_assets *tx, 
    size_t *length, 
    dmbc_error *error
);

/*
    Transfer
*/
dmbc_tx_transfer *dmbc_tx_transfer_create(
    const char *from, 
    const char *to, 
    uint64_t amount,
    uint64_t seed, 
    const char *memo,
    dmbc_error *error
);

void dmbc_tx_transfer_free(dmbc_tx_transfer *tx);

bool dmbc_tx_transfer_add_asset(
    dmbc_tx_transfer *tx,
    dmbc_asset *asset,
    dmbc_error *error
);

uint8_t *dmbc_tx_transfer_into_bytes(
    dmbc_tx_transfer *tx,
    size_t *length,
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

dmbc_tx_exchange *dmbc_tx_exchange_create(
    dmbc_exchange_offer *offer,
    const char *signature,
    uint64_t seed,
    const char *memo,
    dmbc_error *error
);

void dmbc_tx_exchange_free(dmbc_tx_exchange *tx);

uint8_t * dmbc_tx_exchange_into_bytes(
    dmbc_tx_exchange *tx,
    size_t *length,
    dmbc_error *error
);

/*
    Exchnage Intermediary offer
*/
dmbc_exchange_offer_intermediary *dmbc_exchange_offer_intermediary_create(
    dmbc_intermediary *intermediary,
    const char *sender_key,
    uint64_t sender_amount,
    const char *recipient_key,
    u_int8_t fee_strategy,
    dmbc_error *error
);

void dmbc_exchange_offer_intermediary_free(dmbc_exchange_offer_intermediary *offer);

bool dmbc_exchange_offer_intermediary_recipient_add_asset(
    dmbc_exchange_offer_intermediary *offer,
    dmbc_asset *asset,
    dmbc_error *error
);

bool dmbc_exchange_offer_intermediary_sender_add_asset(
    dmbc_exchange_offer_intermediary *offer,
    dmbc_asset *asset,
    dmbc_error *error
);

uint8_t* dmbc_exchange_offer_intermediary_into_bytes(
    dmbc_exchange_offer_intermediary *offer,
    size_t *length,
    dmbc_error *error
);

dmbc_tx_exchange_intermediary *dmbc_tx_exchange_intermediary_create(
    dmbc_exchange_offer_intermediary *offer,
    const char *sender_signature,
    const char *intermediary_signature,
    uint64_t seed,
    const char *memo,
    dmbc_error *error
);

void dmbc_tx_exchange_intermediary_free(dmbc_tx_exchange_intermediary *tx);

uint8_t * dmbc_tx_exchange_intermediary_into_bytes(
    dmbc_tx_exchange_intermediary *tx,
    size_t *length,
    dmbc_error *error
);

/*
    Trade
*/
dmbc_trade_offer *dmbc_trade_offer_create(
    const char *seller_key,
    const char *buyer_key,
    u_int8_t fee_strategy,
    dmbc_error *error
);

void dmbc_trade_offer_free(dmbc_trade_offer *offer);

bool dmbc_trade_offer_add_asset(
    dmbc_trade_offer *offer,
    dmbc_trade_asset *asset,
    dmbc_error *error
);

uint8_t *dmbc_trade_offer_into_bytes(
    dmbc_trade_offer *offer,
    size_t *length,
    dmbc_error *error
);

dmbc_tx_trade *dmbc_tx_trade_create(
    dmbc_trade_offer *offer,
    const char *seller_signature,
    uint64_t seed,
    dmbc_error *error
);

void dmbc_tx_trade_free(dmbc_tx_trade *tx);

uint8_t *dmbc_tx_trade_into_bytes(
    dmbc_tx_trade *tx,
    size_t *length,
    dmbc_error *error
);


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
    Trade Asset
*/
dmbc_trade_asset *dmbc_trade_asset_create(
    const char *id,
    uint64_t amount,
    uint64_t price,
    dmbc_error *error
);

void dmbc_trade_asset_free(dmbc_trade_asset *asset);

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

#ifdef __cplusplus
}
#endif

#endif