#include <stdarg.h>

#include "redismodule.h"

int ExportedRedisModule_Init(RedisModuleCtx *ctx, const char *name, int ver, int apiver) {
    return RedisModule_Init(ctx, name, ver, apiver);
}

void ExportedRedisModule_EmitAOF(RedisModuleIO *aof, const char *cmdname, const char *fmt, va_list args) {
    return RedisModule_EmitAOF(aof, cmdname, fmt, args);
}
