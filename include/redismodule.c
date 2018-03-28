#include "redismodule.h"

int ExportedRedisModule_Init(RedisModuleCtx *ctx, const char *name, int ver, int apiver) {
    return RedisModule_Init(ctx, name, ver, apiver);
}
