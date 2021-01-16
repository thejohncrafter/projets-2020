
#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>
#include <math.h>

#define TYPE_NOTHING 0
#define TYPE_INT64 1
#define TYPE_BOOL 2
#define TYPE_STR 3

// Assuming a 64-bits machine, we get 64 bits long pointers.

void native_print_bool(uint64_t *ret_ty, uint64_t *ret_val, uint64_t ty, uint64_t val) {
    *ret_ty = TYPE_NOTHING;
    *ret_val = 0;

    if(ty != TYPE_BOOL) {
        fprintf(stderr, "Expected a Bool for print_bool.\n");
        exit(1);
    }

    if(val)
        printf("true\n");
    else
        printf("false\n");
}

void native_print_int(uint64_t *ret_ty, uint64_t *ret_val, uint64_t ty, uint64_t val) {
    *ret_ty = TYPE_NOTHING;
    *ret_val = 0;

    if(ty != TYPE_INT64) {
        fprintf(stderr, "Expected a Int64 for print_int.\n");
        exit(1);
    }

    printf("%" PRIu64 "\n", val);
}

void native_print_string(uint64_t *ret_ty, uint64_t *ret_val, uint64_t ty, char* val) {
    *ret_ty = TYPE_NOTHING;
    *ret_val = 0;

    if(ty != TYPE_STR) {
        fprintf(stderr, "Expected a Str for print_string.\n");
        exit(1);
    }

    printf("%s\n", val);
}

void native_pow(uint64_t *ret_ty, uint64_t *ret_val, uint64_t ty_1, int64_t val_1, uint64_t ty_2, int64_t val_2) {
    if(ty_1 != TYPE_INT64 && ty_2 == TYPE_INT64) {
        fprintf(stderr, "Expected two Int64 for pow.\n");
        exit(1);
    }

    *ret_ty = TYPE_INT64;
    *ret_val = pow(val_1, val_2);
}

