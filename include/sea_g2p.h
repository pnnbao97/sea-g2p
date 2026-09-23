/* sea-g2p — Vietnamese/English text to phonemes, for C and C++ hosts.
 *
 * The same two stages the Python package exposes:
 *   - normalisation rewrites raw text into something pronounceable (numbers,
 *     dates, units, abbreviations, formulas, URLs);
 *   - G2P maps the normalised text to phonemes, resolving Vietnamese and
 *     English readings for the same token from context.
 *
 * Build the library from the crate:
 *
 *     cargo build --release --no-default-features --features capi
 *     # target/release/{libsea_g2p_rs.so | sea_g2p_rs.dll | libsea_g2p_rs.dylib}
 *
 * or take it from a GitHub release. Link against it, or load it at runtime —
 * the symbols below are the whole surface.
 *
 * Contract:
 *   - strings are UTF-8 in both directions;
 *   - every returned char* is owned by the caller and freed with
 *     sea_g2p_string_free();
 *   - a failing call returns NULL and leaves a message in sea_g2p_last_error(),
 *     which is per-thread;
 *   - a handle is not thread-safe on its own; separate handles may be used from
 *     separate threads;
 *   - Rust panics are caught at the boundary, never unwound into C.
 *
 * Licence: Apache-2.0, same as the crate.
 */
#ifndef SEA_G2P_H
#define SEA_G2P_H

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque: a loaded dictionary plus the two stages that use it. */
typedef struct SeaG2p sea_g2p;

/* The ABI this header describes. A host should compare it with
 * sea_g2p_abi_version() and refuse a library that reports something else. */
#define SEA_G2P_ABI_VERSION 1
int sea_g2p_abi_version(void);

/* The detail for the last failing call on this thread, or NULL after a
 * successful one. Borrowed: valid until the next failing call on this thread. */
const char *sea_g2p_last_error(void);

/* Opens `sea_g2p.bin` (the dictionary shipped with the Python package) and
 * returns a handle, or NULL on failure. */
sea_g2p *sea_g2p_open(const char *dict_path);

/* Frees a handle. Freeing NULL is a no-op. */
void sea_g2p_close(sea_g2p *handle);

/* Frees a string returned by this library. Freeing NULL is a no-op. */
void sea_g2p_string_free(char *text);

/* Text -> phonemes: normalise, then phonemise. `punc_norm` non-zero applies the
 * trailing-punctuation rule first (a sentence under five words is forced to end
 * in exactly one "."; a longer one gets "." appended when it lacks terminal
 * , . ! ?). */
char *sea_g2p_phonemize(const sea_g2p *handle, const char *text, int punc_norm);

/* Normalisation alone — the length a chunker should measure, because what
 * matters is the length after "3,5 triệu" has become words. */
char *sea_g2p_normalize(const sea_g2p *handle, const char *text, int punc_norm);

/* The trailing-punctuation rule as a pure string operation, for settling a
 * chunk boundary without normalising again. Needs no handle. */
char *sea_g2p_punc_norm(const char *text);

#ifdef __cplusplus
}  /* extern "C" */
#endif

#endif /* SEA_G2P_H */
