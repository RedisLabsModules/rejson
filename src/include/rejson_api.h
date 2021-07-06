#pragma once

#include "redismodule.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum JSONType {
    JSONType_String = 0,
    JSONType_Int = 1,
    JSONType_Double = 2,
    JSONType_Bool = 3,
    JSONType_Object = 4,
    JSONType_Array = 5,
    JSONType_Null = 6,
    JSONType__EOF
} JSONType;

typedef const void* RedisJSON;
typedef const void* JSONResultsIterator;

typedef struct RedisJSONAPI_V1 {
  /* RedisJSONKey functions */
  RedisJSON (*openKey)(RedisModuleCtx *ctx, RedisModuleString *key_name);
  RedisJSON (*openKeyFromStr)(RedisModuleCtx *ctx, const char *path);

  ResultsIterator (*get)(RedisJSON* json, const char *path);
  
  RedisJSON (*next)(ResultsIterator* iter);
  size_t (*len)(ResultsIterator* iter);
  void (*freeIter)(ResultsIterator* iter);

  RedisJSON (*getAt)(RedisJSON json, size_t index);

  /* RedisJSON value functions
   * Return REDISMODULE_OK if RedisJSON is of the correct JSONType,
   * else REDISMODULE_ERR is returned
   * */

  // Return the length of Object/Array
  int (*getLen)(RedisJSON json, size_t *count);

  // Return the JSONType
  JSONType (*getType)(RedisJSON json);

  // Return int value from a Numeric field
  int (*getInt)(RedisJSON json, long long *integer);

  // Return double value from a Numeric field
  int (*getDouble)(RedisJSON json, double *dbl);

  // Return 0 or 1 as int value from a Bool field
  int (*getBoolean)(RedisJSON json, int *boolean);

  // Return a Read-Only String value from a String field
  int (*getString)(RedisJSON json, const char **str, size_t *len);

  // Return JSON String representation (for any JSONType)
  // The caller gains ownership of `str`
  int (*getJSON)(RedisJSON json, RedisModuleCtx *ctx, RedisModuleString **str);

  // Return 1 if type of key is JSON
  int (*isJSON)(RedisModuleKey *redis_key);

} RedisJSONAPI_V1;

#ifdef __cplusplus
}
#endif

