#ifndef _DMBC_CAPI_H
#define _DMBC_CAPI_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct dmbc_builder dmbc_builder;

typedef struct dmbc_error dmbc_error;

dmbc_builder *dmbc_builder_create(
    const char *public_key, 
    const char *private_key, 
    unsigned char network_id,
    unsigned char protocol_version,
    unsigned short message_type,
    dmbc_error *error);

void dmbc_builder_free(dmbc_builder *builder);

dmbc_error *dmbc_error_new();

const char *dmbc_error_message(dmbc_error *error);

void dmbc_error_free(dmbc_error *error);


#ifdef __cplusplus
}
#endif

#endif