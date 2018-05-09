#ifndef _DMBC_CAPI_H
#define _DMBC_CAPI_H

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct dmbc_builder dmbc_builder;

typedef struct dmbc_error dmbc_error;

dmbc_builder *dmbc_builder_create(
    unsigned char network_id,
    unsigned char protocol_version,
    unsigned short service_id,
    unsigned short message_type,
    dmbc_error *error);

void dmbc_builder_free(dmbc_builder *builder);

bool dmbc_add_assets_set_public_key(
    dmbc_builder *builder, 
    const char *public_key, 
    dmbc_error *error);

dmbc_error *dmbc_error_new();

const char *dmbc_error_message(dmbc_error *error);

void dmbc_error_free(dmbc_error *error);

void debug(dmbc_builder *builder);

#ifdef __cplusplus
}
#endif

#endif