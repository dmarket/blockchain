#ifndef _DMBC_CAPI_H
#define _DMBC_CAPI_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * dmbc_tx_transfer is a type of transfer transaction
 */
typedef struct dmbc_tx_transfer dmbc_tx_transfer;

/*
 * dmbc_tx_transfer_fees_payer_offer is a type of transfer offer with fees payer
 */
typedef struct dmbc_transfer_fees_payer_offer dmbc_transfer_fees_payer_offer;

/*
 * dmbc_tx_transfer_fees_payer is a type of transfer transaction with feees payer
 */
typedef struct dmbc_tx_transfer_fees_payer dmbc_tx_transfer_fees_payer;

/*
 * dmbc_tx_add_assets is a type of add_assets transaction
 */
typedef struct dmbc_tx_add_assets dmbc_tx_add_assets;

/*
 * dmbc_tx_delete_assets is a type of delete_assets transaction
 */
typedef struct dmbc_tx_delete_assets dmbc_tx_delete_assets;

/*
 * dmbc_asset is a type of asset bundle object
 */
typedef struct dmbc_asset dmbc_asset;

/*
 * dmbc_trade_asset is a type of trade asset object
 */
typedef struct dmbc_trade_asset dmbc_trade_asset;

/*
 * dmbc_fees is a type of fees object
 */
typedef struct dmbc_fees dmbc_fees;

/*
 * dmbc_intermediary is a type of intermediary object
 */
typedef struct dmbc_intermediary dmbc_intermediary;

/*
 * dmbc_exchange_offer is a type of exchange offer object
 */
typedef struct dmbc_exchange_offer dmbc_exchange_offer;

/*
 * dmbc_tx_exchange is a type of exchange transaction
 */
typedef struct dmbc_tx_exchange dmbc_tx_exchange;

/*
 * dmbc_exchange_offer_intermediary is a type of exchange with intermediary object
 */
typedef struct dmbc_exchange_offer_intermediary dmbc_exchange_offer_intermediary;

/*
 * dmbc_exchange_offer_intermediary is a type of exchange with intermediary transaction
 */
typedef struct dmbc_tx_exchange_intermediary dmbc_tx_exchange_intermediary;

/*
 * dmbc_trade_offer is a type of trade offer object
 */
typedef struct dmbc_trade_offer dmbc_trade_offer;

/*
 * dmbc_trade_offer_intermediary is a type of trade with intermediary object
 */
typedef struct dmbc_trade_offer_intermediary dmbc_trade_offer_intermediary;

/*
 * dmbc_tx_trade is a type of trade transaction
 */
typedef struct dmbc_tx_trade dmbc_tx_trade;

/*
 * dmbc_tx_trade_intermediary is a type of trade with intermediary transaction
 */
typedef struct dmbc_tx_trade_intermediary dmbc_tx_trade_intermediary;

/*
 * dmbc_tx_ask_offer is a type of ask offer transaction
 */
typedef struct dmbc_tx_ask_offer dmbc_tx_ask_offer;

/*
 * dmbc_tx_bid_offer is a type of bid offer transaction
 */
typedef struct dmbc_tx_bid_offer dmbc_tx_bid_offer;

/*
 * dmbc_error is a type of error object
 */
typedef struct dmbc_error dmbc_error;

/* fee stategy flags */
#define FEE_STRATEGY_RECIPIENT 1
#define FEE_STRATEGY_SENDER 2
#define FEE_STRATEGY_BOTH 3
#define FEE_STRATEGY_INTERMEDIARY 4

/*
 * @dmbc_bytes_free frees the memory allocated on the heap.
 * 
 * @bytes is a pointer to allocated buffer in heap.
 * @length is a size of allocated buffer.
 */
void dmbc_bytes_free(uint8_t *bytes, size_t length);

/*
 * @dmbc_tx_add_assets_create creates add_assets transaction object on the heap.
 * Object should be dealocated when it is no needed anymore.
 * 
 * @public_key public key [32 bytes long] in hex format.
 * @seed transaction seed number.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_tx_add_assets pointer to add_asset transaction, otherwise NULL.
 */
dmbc_tx_add_assets *dmbc_tx_add_assets_create(
    const char *public_key,
    uint64_t seed,
    dmbc_error *error
);

/*
 * @dmbc_tx_add_asset_free frees allocated add_assets transaction.
 * 
 * @dmbc_tx_add_assets pointer to add_asset transaction.
 */
void dmbc_tx_add_asset_free(dmbc_tx_add_assets *tx);

/*
 * @dmbc_tx_add_assets_add_asset adds asset into add_assets transaction.
 * 
 * @tx pointer to add_asset transaction.
 * @name asset's meta data.
 * @amount amount of items.
 * @fees pointer to fees object.
 * @receiver_key receiver's public key [32 bytes long] in hex format.
 * @error contains error message if any occurs.
 * 
 * @ret true if operation succeeded, otherwise false.
*/
bool dmbc_tx_add_assets_add_asset(
    dmbc_tx_add_assets *tx,
    const char *name, 
    uint64_t amount,
    dmbc_fees *fees,
    const char *receiver_key,
    dmbc_error *error
);

/*
 * dmbc_tx_add_assets_into_bytes converts add_assets transaction into byte array.
 * 
 * @tx pointer to add_asset transaction.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t *dmbc_tx_add_assets_into_bytes(
    dmbc_tx_add_assets *tx, 
    size_t *length, 
    dmbc_error *error
);

/*
 * @dmbc_tx_delete_assets_create creates delete_assets transaction object on the heap.
 * Object should be dealocated when it is no needed anymore.
 * 
 * @public_key public key [32 bytes long] in hex format.
 * @seed transaction seed number.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_tx_delete_assets pointer to delete_asset transaction, otherwise NULL.
 */
dmbc_tx_delete_assets *dmbc_tx_delete_assets_create(
    const char *public_key,
    uint64_t seed,
    dmbc_error *error
);

/*
 * @dmbc_tx_delete_assets_free frees allocated delete_asset transaction.
 * 
 * @dmbc_tx_delete_assets pointer to delete_asset transaction.
 */
void dmbc_tx_delete_assets_free(dmbc_tx_delete_assets *tx);

/*
 * @dmbc_tx_delete_assets_add_asset adds asset into delete_assets transaction.
 * 
 * @tx pointer to delete_assets transaction.
 * @asset pointer to asset object.
 * @error contains error message if any occurs.
 * 
 * @ret true if operation succeeded, otherwise false.
*/
bool dmbc_tx_delete_assets_add_asset(
    dmbc_tx_delete_assets *tx,
    dmbc_asset *asset,
    dmbc_error *error
);

/*
 * dmbc_tx_delete_assets_into_bytes converts delete_assets transaction into byte array.
 * 
 * @tx pointer to delete_assets transaction.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t *dmbc_tx_delete_assets_into_bytes(
    dmbc_tx_delete_assets *tx, 
    size_t *length, 
    dmbc_error *error
);

/*
 * @dmbc_tx_transfer_create creates transaction transaction object on the heap.
 * Object should be dealocated when it is no needed anymore.
 * 
 * @from public key of a sender [32 bytes long] in hex format.
 * @to public key of a receiver [32 bytes long] in hex format.
 * @amount coins value.
 * @seed transaction seed number.
 * @memo memo messsage.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_tx_transfer pointer to transfer transaction, otherwise NULL.
 */
dmbc_tx_transfer *dmbc_tx_transfer_create(
    const char *from, 
    const char *to, 
    uint64_t amount,
    uint64_t seed, 
    const char *memo,
    dmbc_error *error
);

/*
 * @dmbc_tx_transfer_free frees allocated transfer transaction.
 * 
 * @dmbc_tx_transfer pointer to transfer transaction.
 */
void dmbc_tx_transfer_free(dmbc_tx_transfer *tx);

/*
 * @dmbc_tx_transfer_add_asset adds asset into transfer transaction.
 * 
 * @tx pointer to dmbc_tx_transfer transaction.
 * @asset pointer to asset object.
 * @error contains error message if any occurs.
 * 
 * @ret true if operation succeeded, otherwise false.
*/
bool dmbc_tx_transfer_add_asset(
    dmbc_tx_transfer *tx,
    dmbc_asset *asset,
    dmbc_error *error
);

/*
 * dmbc_tx_transfer_into_bytes converts transfer transaction into byte array.
 * 
 * @tx pointer to transfer transaction.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t *dmbc_tx_transfer_into_bytes(
    dmbc_tx_transfer *tx,
    size_t *length,
    dmbc_error *error
);

/*
 * @dmbc_transfer_fees_payer_offer_create creates transfer fees payer offer object on the heap.
 * Object should be dealocated when it is no needed anymore.
 * 
 * @from_key public key of a sender [32 bytes long] in hex format.
 * @to_key public key of a receiver [32 bytes long] in hex format.
 * @fees_payer_key public key of a fees payer [32 bytes long] in hex format.
 * @amount amount of coins from sender.
 * @seed seed number.
 * @data_info memo message.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_transfer_fees_payer_offer pointer to transfer fees payer offer object, otherwise NULL.
 */
dmbc_transfer_fees_payer_offer *dmbc_transfer_fees_payer_offer_create(
    const char *from_key,
    const char *to_key,
    const char *fees_payer_key,
    uint64_t amount,
    uint64_t seed,
    const char *data_info,
    dmbc_error *error
);

/*
 * @dmbc_transfer_fees_payer_offer_free frees allocated transfer offer object.
 * 
 * @dmbc_transfer_fees_payer_offer pointer to transfer offer object.
 */
void dmbc_transfer_fees_payer_offer_free(dmbc_transfer_fees_payer_offer *offer);

/*
 * @dmbc_transfer_fees_payer_offer_add_asset adds sender's asset to offer.
 * 
 * @offer pointer to dmbc_transfer_fees_payer_offer offer object.
 * @asset pointer to asset object.
 * @error contains error message if any occurs.
 */
bool dmbc_transfer_fees_payer_offer_add_asset(
    dmbc_transfer_fees_payer_offer *offer,
    dmbc_asset *asset,
    dmbc_error *error
);

/*
 * dmbc_transfer_fees_payer_offer_into_bytes converts offer object into byte array.
 * 
 * @offer pointer to offer object.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t* dmbc_transfer_fees_payer_offer_into_bytes(
    dmbc_transfer_fees_payer_offer *offer,
    size_t *length,
    dmbc_error *error
);

/*
 * @dmbc_tx_transfer_fees_payer_create creates transfer fees payer transaction object on the heap.
 * Object should be dealocated when it is no needed anymore.
 * 
 * @offer pointer to transfer offer object.
 * @signature signature of an offer signed by fees payer [64 bytes long] in hex format.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_tx_transfer_fees_payer pointer to exchange transaction, otherwise NULL.
 */
dmbc_tx_transfer_fees_payer *dmbc_tx_transfer_fees_payer_create(
    dmbc_transfer_fees_payer_offer *offer,
    const char *fees_payer_signature,
    dmbc_error *error
);

/*
 * @dmbc_tx_transfer_fees_payer_free frees allocated transfer transaction.
 * 
 * @dmbc_tx_transfer_fees_payer pointer to transfer fees payer transaction.
 */
void dmbc_tx_transfer_fees_payer_free(dmbc_tx_transfer_fees_payer *tx);

/*
 * dmbc_tx_transfer_fees_payer_into_bytes converts transaction into byte array.
 * 
 * @tx pointer to transation.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t * dmbc_tx_transfer_fees_payer_into_bytes(
    dmbc_tx_transfer_fees_payer *tx,
    size_t *length,
    dmbc_error *error
);

/*
 * @dmbc_exchange_offer_create creates exchange offer object on the heap.
 * Object should be dealocated when it is no needed anymore.
 * 
 * @sender_key public key of a sender [32 bytes long] in hex format.
 * @sender_amount amount of coins from sender.
 * @recipient_key public key of a receiver [32 bytes long] in hex format.
 * @fee_strategy fee strategy flag.
 * @seed transaction seed number.
 * @memo memo messsage.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_exchange_offer pointer to exchange offer object, otherwise NULL.
 */
dmbc_exchange_offer *dmbc_exchange_offer_create(
    const char *sender_key,
    uint64_t sender_amount,
    const char *recipient_key,
    uint8_t fee_strategy,
    uint64_t seed,
    const char *memo,
    dmbc_error *error
);

/*
 * @dmbc_exchange_offer_free frees allocated exchange offer object.
 * 
 * @dmbc_exchange_offer pointer to exchange offer object.
 */
void dmbc_exchange_offer_free(dmbc_exchange_offer *offer);

/*
 * @dmbc_exchange_offer_recipient_add_asset adds recipient's
 *  asset into exchange offer object.
 * 
 * @offer pointer to dmbc_exchange_offer object.
 * @asset pointer to asset object.
 * @error contains error message if any occurs.
 * 
 * @ret true if operation succeeded, otherwise false.
*/
bool dmbc_exchange_offer_recipient_add_asset(
    dmbc_exchange_offer *offer,
    dmbc_asset *asset,
    dmbc_error *error
);

/*
 * @dmbc_exchange_offer_sender_add_asset adds sender's
 *  asset into exchange offer object.
 * 
 * @offer pointer to dmbc_exchange_offer object.
 * @asset pointer to asset object.
 * @error contains error message if any occurs.
 * 
 * @ret true if operation succeeded, otherwise false.
*/
bool dmbc_exchange_offer_sender_add_asset(
    dmbc_exchange_offer *offer,
    dmbc_asset *asset,
    dmbc_error *error
);

/*
 * dmbc_exchange_offer_into_bytes converts offer object into byte array.
 * 
 * @offer pointer to offer object.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t* dmbc_exchange_offer_into_bytes(
    dmbc_exchange_offer *offer,
    size_t *length,
    dmbc_error *error
);

/*
 * @dmbc_tx_exchange_create creates exchange transaction object on the heap.
 * Object should be dealocated when it is no needed anymore.
 * 
 * @offer pointer to exchange offer object.
 * @signature signature of an offer [64 bytes long] in hex format.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_tx_exchange pointer to exchange transaction, otherwise NULL.
 */
dmbc_tx_exchange *dmbc_tx_exchange_create(
    dmbc_exchange_offer *offer,
    const char *signature,
    dmbc_error *error
);

/*
 * @dmbc_tx_exchange_free frees allocated exchange transaction.
 * 
 * @dmbc_tx_exchange pointer to exchange transaction.
 */
void dmbc_tx_exchange_free(dmbc_tx_exchange *tx);

/*
 * dmbc_tx_exchange_into_bytes converts transaction into byte array.
 * 
 * @tx pointer to transation.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t * dmbc_tx_exchange_into_bytes(
    dmbc_tx_exchange *tx,
    size_t *length,
    dmbc_error *error
);

/*
 * @dmbc_exchange_offer_intermediary_create creates exchange 
 * offer with intermediary object on the heap.
 * Object should be dealocated when it is no needed anymore.
 * 
 * @intermediary pointer to intermediary object.
 * @sender_key public key of a sender [32 bytes long] in hex format.
 * @sender_amount amount of coins from sender.
 * @recipient_key public key of a receiver [32 bytes long] in hex format.
 * @fee_strategy fee strategy flag.
 * @seed transaction seed number.
 * @memo memo messsage.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_exchange_offer_intermediary pointer to exchange offer object,
 *  otherwise NULL.
 */
dmbc_exchange_offer_intermediary *dmbc_exchange_offer_intermediary_create(
    dmbc_intermediary *intermediary,
    const char *sender_key,
    uint64_t sender_amount,
    const char *recipient_key,
    uint8_t fee_strategy,
    uint64_t seed,
    const char *memo,
    dmbc_error *error
);

/*
 * @dmbc_exchange_offer_intermediary_free frees allocated exchange offer
 *  with intermediary object.
 * 
 * @dmbc_exchange_offer_intermediary pointer to exchange offer object.
 */
void dmbc_exchange_offer_intermediary_free(dmbc_exchange_offer_intermediary *offer);

/*
 * @dmbc_exchange_offer_intermediary_recipient_add_asset adds recipient's
 *  asset into exchange offer object.
 * 
 * @offer pointer to dmbc_exchange_offer_intermediary object.
 * @asset pointer to asset object.
 * @error contains error message if any occurs.
 * 
 * @ret true if operation succeeded, otherwise false.
*/
bool dmbc_exchange_offer_intermediary_recipient_add_asset(
    dmbc_exchange_offer_intermediary *offer,
    dmbc_asset *asset,
    dmbc_error *error
);

/*
 * @dmbc_exchange_offer_intermediary_sender_add_asset adds sender's
 *  asset into exchange offer object.
 * 
 * @offer pointer to dmbc_exchange_offer_intermediary object.
 * @asset pointer to asset object.
 * @error contains error message if any occurs.
 * 
 * @ret true if operation succeeded, otherwise false.
*/
bool dmbc_exchange_offer_intermediary_sender_add_asset(
    dmbc_exchange_offer_intermediary *offer,
    dmbc_asset *asset,
    dmbc_error *error
);

/*
 * dmbc_exchange_offer_intermediary_into_bytes converts exchange 
 * offer into byte array.
 * 
 * @offer pointer to offer.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t* dmbc_exchange_offer_intermediary_into_bytes(
    dmbc_exchange_offer_intermediary *offer,
    size_t *length,
    dmbc_error *error
);

/*
 * @dmbc_tx_exchange_intermediary_create creates exchange transaction with intermediary 
 * object on the heap.
 * Object should be dealocated when it is no needed anymore.
 * 
 * @offer pointer to exchange offer with intermediary object.
 * @sender_signature signature of an offer [64 bytes long] in hex format signed by sender.
 * @intermediary_signature signature of an offer [64 bytes long] in hex format signed by intermediary party.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_tx_exchange_intermediary pointer to exchange transaction, otherwise NULL.
 */
dmbc_tx_exchange_intermediary *dmbc_tx_exchange_intermediary_create(
    dmbc_exchange_offer_intermediary *offer,
    const char *sender_signature,
    const char *intermediary_signature,
    dmbc_error *error
);

/*
 * @dmbc_tx_exchange_intermediary_free frees allocated exchange transaction.
 * 
 * @dmbc_tx_exchange_intermediary pointer to exchange transaction.
 */
void dmbc_tx_exchange_intermediary_free(dmbc_tx_exchange_intermediary *tx);

/*
 * dmbc_tx_exchange_intermediary_into_bytes converts transaction into byte array.
 * 
 * @tx pointer to transation.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t * dmbc_tx_exchange_intermediary_into_bytes(
    dmbc_tx_exchange_intermediary *tx,
    size_t *length,
    dmbc_error *error
);

/*
 * @dmbc_trade_offer_create creates trade offer object on the heap.
 * Object should be dealocated when it is no needed anymore.
 * 
 * @seller_key public key of a seller [32 bytes long] in hex format.
 * @buyer_key public key of a buyer [32 bytes long] in hex format.
 * @fee_strategy fee strategy flag.
 * @seed transaction seed number.
 * @data_info memo message.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_trade_offer pointer to trade offer object, otherwise NULL.
 */
dmbc_trade_offer *dmbc_trade_offer_create(
    const char *seller_key,
    const char *buyer_key,
    uint8_t fee_strategy,
    uint64_t seed,
    const char *data_info,
    dmbc_error *error
);

/*
 * @dmbc_trade_offer_free frees allocated trade offer.
 * 
 * @dmbc_trade_offer pointer to trade offer object.
 */
void dmbc_trade_offer_free(dmbc_trade_offer *offer);

/*
 * @dmbc_trade_offer_add_asset adds
 *  asset into trade offer object.
 * 
 * @offer pointer to dmbc_trade_offer object.
 * @asset pointer to trade asset object.
 * @error contains error message if any occurs.
 * 
 * @ret true if operation succeeded, otherwise false.
*/
bool dmbc_trade_offer_add_asset(
    dmbc_trade_offer *offer,
    dmbc_trade_asset *asset,
    dmbc_error *error
);

/*
 * dmbc_trade_offer_into_bytes converts trade 
 * offer into byte array.
 * 
 * @offer pointer to offer.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t *dmbc_trade_offer_into_bytes(
    dmbc_trade_offer *offer,
    size_t *length,
    dmbc_error *error
);

/*
 * @dmbc_tx_trade_create creates trade transaction 
 * object on the heap.
 * Object should be dealocated when it is no needed anymore.
 * 
 * @offer pointer to trade offer object.
 * @seller_signature signature of a seller [64 bytes long] in hex format signed by sender.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_tx_trade pointer to exchange transaction, otherwise NULL.
 */
dmbc_tx_trade *dmbc_tx_trade_create(
    dmbc_trade_offer *offer,
    const char *seller_signature,
    dmbc_error *error
);

/*
 * @dmbc_tx_trade_free frees allocated trade transaction.
 * 
 * @dmbc_tx_trade pointer to trade transaction.
 */
void dmbc_tx_trade_free(dmbc_tx_trade *tx);

/*
 * dmbc_tx_trade_into_bytes converts transaction into byte array.
 * 
 * @tx pointer to transation.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t *dmbc_tx_trade_into_bytes(
    dmbc_tx_trade *tx,
    size_t *length,
    dmbc_error *error
);

/*
 * @dmbc_trade_offer_intermediary_create creates trade offer object on the heap.
 * Object should be dealocated when it is no needed anymore.
 * 
 * @intermediary pointer to intermediary object.
 * @seller_key public key of a seller [32 bytes long] in hex format.
 * @buyer_key public key of a buyer [32 bytes long] in hex format.
 * @fee_strategy fee strategy flag.
 * @seed transaction seed number.
 * @memo memo message.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_trade_offer_intermediary pointer to trade offer object, otherwise NULL.
 */
dmbc_trade_offer_intermediary *dmbc_trade_offer_intermediary_create(
    dmbc_intermediary *intermediary,
    const char *seller_key,
    const char *buyer_key,
    uint8_t fee_strategy,
    uint64_t seed,
    const char *memo,
    dmbc_error *error
);

/*
 * @dmbc_trade_offer_intermediary_free frees allocated trade offer.
 * 
 * @dmbc_trade_offer_intermediary pointer to trade offer object.
 */
void dmbc_trade_offer_intermediary_free(dmbc_trade_offer_intermediary *offer);

/*
 * @dmbc_trade_offer_intermediary_add_asset adds
 *  asset into trade offer object.
 * 
 * @offer pointer to dmbc_trade_offer_intermediary object.
 * @asset pointer to trade asset object.
 * @error contains error message if any occurs.
 * 
 * @ret true if operation succeeded, otherwise false.
*/
bool dmbc_trade_offer_intermediary_add_asset(
    dmbc_trade_offer_intermediary *offer,
    dmbc_trade_asset *asset,
    dmbc_error *error
);

/*
 * dmbc_trade_offer_intermediary_into_bytes converts trade 
 * offer into byte array.
 * 
 * @offer pointer to offer.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t *dmbc_trade_offer_intermediary_into_bytes(
    dmbc_trade_offer_intermediary *offer,
    size_t *length,
    dmbc_error *error
);

/*
 * @dmbc_tx_trade_intermediary_create creates trade transaction 
 * with intermediary object on the heap.
 * Object should be dealocated when it is no needed anymore.
 * 
 * @offer pointer to trade offer object.
 * @seller_signature signature [64 bytes long] in hex format signed by seller.
 * @intermediary_signature signature [64 bytes long] in hex format signed by internediary party.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_tx_trade_intermediary pointer to trade transaction, otherwise NULL.
 */
dmbc_tx_trade_intermediary *dmbc_tx_trade_intermediary_create(
    dmbc_trade_offer_intermediary *offer,
    const char *seller_signature,
    const char *intermediary_signature,
    dmbc_error *error
);

/*
 * @dmbc_tx_trade_intermediary_free frees allocated trade transaction.
 * 
 * @dmbc_tx_trade_intermediary pointer to trade transaction.
 */
void dmbc_tx_trade_intermediary_free(dmbc_tx_trade_intermediary *tx);

/*
 * dmbc_tx_trade_intermediary_into_bytes converts transaction into byte array.
 * 
 * @tx pointer to transation.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t *dmbc_tx_trade_intermediary_into_bytes(
    dmbc_tx_trade_intermediary *tx,
    size_t *length,
    dmbc_error *error
);

/*
 * dmbc_tx_ask_offer_create creates ask offer transaction object.
 * 
 * @public_key public key of a seller [32 bytes long] in hex format.
 * @asset pointer to trade asset object.
 * @error contains error message if any occurs.
 * 
 * @ret pointer to dmbc_tx_ask_offer if succeeded, otherwise NULL.
 */
dmbc_tx_ask_offer *dmbc_tx_ask_offer_create(
    const char *public_key,
    dmbc_trade_asset *asset,
    uint64_t seed,
    const char *data_info,
    dmbc_error *error
);

/*
 * @dmbc_tx_ask_offer_free frees allocated ask offer transaction.
 * 
 * @dmbc_tx_ask_offer_free pointer to ask offer transaction.
 */
void dmbc_tx_ask_offer_free(dmbc_tx_ask_offer *tx);


/*
 * dmbc_tx_ask_offer_into_bytes converts transaction into byte array.
 * 
 * @tx pointer to transation.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t *dmbc_tx_ask_offer_into_bytes(
    dmbc_tx_ask_offer *tx,
    size_t *length,
    dmbc_error *error
);

/*
 * dmbc_tx_bid_offer_create creates bid offer transaction object.
 * 
 * @public_key public key of a seller [32 bytes long] in hex format.
 * @asset pointer to trade asset object.
 * @error contains error message if any occurs.
 * 
 * @ret pointer to dmbc_tx_bid_offer if succeeded, otherwise NULL.
 */
dmbc_tx_bid_offer *dmbc_tx_bid_offer_create(
    const char *public_key,
    dmbc_trade_asset *asset,
    uint64_t seed,
    const char *data_info,
    dmbc_error *error
);

/*
 * @dmbc_tx_bid_offer_free frees allocated bid offer transaction.
 * 
 * @dmbc_tx_bid_offer_free pointer to bid offer transaction.
 */
void dmbc_tx_bid_offer_free(dmbc_tx_bid_offer *tx);

/*
 * dmbc_tx_bid_offer_into_bytes converts transaction into byte array.
 * 
 * @tx pointer to transation.
 * @length output parameter, contains byte array size.
 * @error contains error message if any occurs.
 * 
 * @ret byte array if succeeded, otherwise NULL.
 */
uint8_t *dmbc_tx_bid_offer_into_bytes(
    dmbc_tx_bid_offer *tx,
    size_t *length,
    dmbc_error *error
);

/*
 * dmbc_asset_create creates asset bundle object on the heap.
 * 
 * @id asset's id [32 bytes long] in hex.
 * @amount amount of items in asset.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_asset pointer if succeeded, otherwise NULL.
 */
dmbc_asset *dmbc_asset_create(
    const char *id,
    uint64_t amount,
    dmbc_error *error
);

/*
 * @dmbc_asset_free frees allocated asset bundle object.
 * 
 * @asset pointer to asset bundle.
 */
void dmbc_asset_free(dmbc_asset *asset);

/*
 * dmbc_trade_asset_create creates trade asset object on the heap.
 * 
 * @id asset's id [32 bytes long] in hex.
 * @amount amount of items in asset.
 * @price price of the asset item.
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_trade_asset pointer if succeeded, otherwise NULL.
 */
dmbc_trade_asset *dmbc_trade_asset_create(
    const char *id,
    uint64_t amount,
    uint64_t price,
    dmbc_error *error
);

/*
 * @dmbc_trade_asset_free frees allocated asset object.
 * 
 * @asset pointer to trade asset.
 */
void dmbc_trade_asset_free(dmbc_trade_asset *asset);

/*
 * dmbc_fees_create creates fees asset object on the heap.
 * 
 * @trade_fixed trade fee fixed part.
 * @trade_fraction trade fee fraction part. "0.1929391929391"
 * @exchange_fixed exchange fee fixed part.
 * @exchange_fraction exchange fee fraction part. "0.1929391929391"
 * @transfer_fixed transfer fee fixed part.
 * @transfer_fraction transfer fee fraction part. "0.1929391929391"
 * @error contains error message if any occurs.
 * 
 * @ret dmbc_fees pointer if succeeded, otherwise NULL.
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

/*
 * @dmbc_fees_free frees allocated fees object.
 * 
 * @fees pointer to fees object.
 */
void dmbc_fees_free(dmbc_fees *fees);

/*
 * dmbc_intermediary_create creates intermediary object on the heap.
 * 
 * @public_key public key of an intermediary party [32 bytes long] in hex format.
 * @commission intermediary's commission.
 *
 * @ret dmbc_intermediary pointer if succeeded, otherwise NULL.
 */
dmbc_intermediary *dmbc_intermediary_create(
    const char *public_key,
    uint64_t commission
);

/*
 * @dmbc_intermediary_free frees allocated intermediary object.
 * 
 * @intermediary pointer to intermediary object.
 */
void dmbc_intermediary_free(dmbc_intermediary *intermediary);

/*
 * dmbc_error_new creates error object on the heap.
 *
 * @ret dmbc_error pointer if succeeded, otherwise NULL.
 */
dmbc_error *dmbc_error_new();

/*
 * dmbc_error_message returns error message.
 *
 * @ret error string.
 */
const char *dmbc_error_message(dmbc_error *error);

/*
 * @dmbc_error_free frees allocated error object.
 * 
 * @error pointer to error object.
 */
void dmbc_error_free(dmbc_error *error);

#ifdef __cplusplus
}
#endif

#endif