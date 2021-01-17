
#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>
#include <math.h>

#define TYPE_NOTHING 1
#define TYPE_INT64 2
#define TYPE_BOOL 3
#define TYPE_STR 4

// Assuming a 64-bits machine, we get 64 bits long pointers.

void native_div(int64_t *ret_ty, int64_t *ret_val, int64_t ty_1, int64_t val_1, int64_t ty_2, int64_t val_2) {
    if(ty_1 != TYPE_INT64 && ty_2 == TYPE_INT64) {
        fprintf(stderr, "Expected two Int64 for div.\n");
        exit(1);
    }

    int64_t res = val_1 / val_2;

    *ret_ty = TYPE_INT64;
    *ret_val = res;
}

void native_print_nothing(int64_t *ret_ty, int64_t *ret_val, int64_t ty, int64_t _) {
    if (ty != TYPE_NOTHING) {
        fprintf(stderr, "Expected a Nothing for print_nothing.\n");
        exit(1);
    }

    printf("nothing");

    *ret_ty = TYPE_NOTHING;
    *ret_val = 0;
}

void native_print_bool(int64_t *ret_ty, int64_t *ret_val, int64_t ty, int64_t val) {
    if(ty != TYPE_BOOL) {
        fprintf(stderr, "Expected a Bool for print_bool.\n");
        exit(1);
    }

    if(val)
        printf("true");
    else
        printf("false");

    *ret_ty = TYPE_NOTHING;
    *ret_val = 0;
}

void native_print_int(int64_t *ret_ty, int64_t *ret_val, int64_t ty, int64_t val) {
    if(ty != TYPE_INT64) {
        printf("Got ty = %" PRIi64 "; val = %" PRIi64 "\n", ty, val);
        fprintf(stderr, "Expected a Int64 for print_int.\n");
        exit(1);
    }

    printf("%" PRIi64, val);

    *ret_ty = TYPE_NOTHING;
    *ret_val = 0;
}

void native_print_string(int64_t *ret_ty, int64_t *ret_val, int64_t ty, char* val) {
    if(ty != TYPE_STR) {
        fprintf(stderr, "Expected a Str for print_string.\n");
        exit(1);
    }

    printf("%s", val);

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

void native_alloc(int64_t *ret_ty, int64_t *ret_val, int64_t type_id, int64_t mem_len) {
    // We will trust the caller
    // because every call to this function is done from 

    int64_t* pointer = calloc(mem_len, sizeof(char));

    *ret_ty = type_id;
    *ret_val = (int64_t) pointer;
}

void native_panic(int64_t *ret_ty, int64_t *ret_val, int64_t ty, char* val) {
    if(ty != TYPE_STR) {
        fprintf(stderr, "Expected a Str for panic.\n");
        exit(1);
    }

    printf("%s\n", val);

    exit(1);
}

