if exists('b:current_syntax')
    finish
endif

let b:current_syntax = 'redis'

syn keyword redisKeyword SET GET DEL EXISTS KEYS SCAN EXPIRE TTL PERSIST 
syn keyword redisKeyword RENAME TYPE LPUSH RPUSH LPOP RPOP LLEN LRANGE SADD SREM SMEMBERS 
syn keyword redisKeyword SISMEMBER SMOVE SUNION SDIFF SINTER HSET HGET HDEL HGETALL 
syn keyword redisKeyword HKEYS HVALS INCR DECR INCRBY DECRBY APPEND GETRANGE SETRANGE 
syn keyword redisKeyword MSET MGET PUBLISH SUBSCRIBE PSUBSCRIBE PUNSUBSCRIBE FLUSHDB 
syn keyword redisKeyword FLUSHALL PING ECHO SELECT MOVE AUTH QUIT INFO CONFIG 
syn keyword redisKeyword CLIENT MONITOR SLOWLOG COMMAND ACL

syn region redisString     start=+"+ end=+"+

hi link redisKeyword    Type
hi link redisString     String
