import binascii
import libwrapper
import nacl.signing

# generate key pair
signing_key = nacl.signing.SigningKey.generate()
public_key = binascii.hexlify(bytes(signing_key.verify_key))

print("public key: {}".format(public_key))

# load dmbc-capi library
lib = libwrapper.load_lib()

# create error object
error = lib.dmbc_error_new()

seed = 123
# create add_assets transaction (
tx = lib.dmbc_tx_add_assets_create(public_key, seed, error)

# create fees object
fees = lib.dmbc_fees_create(10, "0.1".encode('ascii'), 20, "0.2".encode('ascii'), 9, "0.99999".encode('ascii'), error)

# pack assets into the transaction
lib.dmbc_tx_add_assets_add_asset(tx, "Asset#10".encode('ascii'), 10, fees, public_key, error)
lib.dmbc_tx_add_assets_add_asset(tx, "Asset#00".encode('ascii'), 10000, fees, public_key, error)

# convert transaction into raw buffer
length = libwrapper.make_size_variable()
raw_buffer = lib.dmbc_tx_add_assets_into_bytes(tx, length, error)

# convert to python compatible byte array type
byte_buffer = libwrapper.to_bytes(raw_buffer, length[0])

# sign the data
signed = signing_key.sign(byte_buffer)
print("signature {}".format(binascii.hexlify(signed.signature)))

# verify signed message
verify_key = signing_key.verify_key
message = verify_key.verify(signed.message, signed.signature)

assert message == byte_buffer

# NOTE: in order to avoid memory leaks all objects received from lib calls
# free raw buffer
lib.dmbc_bytes_free(raw_buffer, length[0])

# free fees
lib.dmbc_fees_free(fees)

# free transaction
lib.dmbc_tx_add_asset_free(tx)

# free error object
lib.dmbc_error_free(error)
