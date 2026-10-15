#ifndef RUSTMAGE_H
#define RUSTMAGE_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

typedef struct RustmageSession RustmageSession;

uint32_t rm_abi_version(void);
/* config_json: NUL-terminated UTF-8 Config. error_out: writable pointer storage.
 * Failure returns NULL and an owned error string. Free it with rm_string_free.
 * On success error_out is set to NULL. */
RustmageSession *rm_new(const char *config_json, char **error_out);
/* Requests/responses are UTF-8 JSON. Result is owned, freed with rm_string_free.
 * On an "engine panic" error, discard this handle without further requests. */
char *rm_request(RustmageSession *session, const char *request_json);
/* Fork current game only. Returns an independently owned handle, or NULL. */
RustmageSession *rm_clone(const RustmageSession *session);
/* NULL is allowed. Otherwise exactly one free per allocated handle/string. */
void rm_free(RustmageSession *session);
void rm_string_free(char *value);

/* Every non-NULL pointer must be valid and live. Do not share a session across
 * concurrent calls. Separate sessions are independent and may run concurrently.
 * Game/snapshot access is privileged and includes hidden information: use
 * player observations to feed policies. No remote security boundary is implied. */
#ifdef __cplusplus
}
#endif
#endif
