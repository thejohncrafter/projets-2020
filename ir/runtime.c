
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

void native_print_bool(int64_t *ret_ty, int64_t *ret_val, int64_t ty, int64_t val) {
    if(ty != TYPE_BOOL) {
        fprintf(stderr, "Expected a Bool for print_bool.\n");
        exit(1);
    }

    if(val)
        printf("true\n");
    else
        printf("false\n");

    *ret_ty = TYPE_NOTHING;
    *ret_val = 0;
}

void native_print_int(int64_t *ret_ty, int64_t *ret_val, int64_t ty, int64_t val) {
    if(ty != TYPE_INT64) {
        fprintf(stderr, "Expected a Int64 for print_int.\n");
        exit(1);
    }

    printf("%" PRIu64 "\n", val);

    *ret_ty = TYPE_NOTHING;
    *ret_val = 0;
}

void native_print_string(int64_t *ret_ty, int64_t *ret_val, int64_t ty, char* val) {
    if(ty != TYPE_STR) {
        fprintf(stderr, "Expected a Str for print_string.\n");
        exit(1);
    }

    printf("%s\n", val);

    *ret_ty = TYPE_NOTHING;
    *ret_val = 0;
}

void native_pow(int64_t *ret_ty, int64_t *ret_val, int64_t ty_1, int64_t val_1, int64_t ty_2, int64_t val_2) {
    if(ty_1 != TYPE_INT64 && ty_2 == TYPE_INT64) {
        fprintf(stderr, "Expected two Int64 for pow.\n");
        exit(1);
    }

    int64_t res = pow(val_1, val_2);

    *ret_ty = TYPE_INT64;
    *ret_val = res;
}

