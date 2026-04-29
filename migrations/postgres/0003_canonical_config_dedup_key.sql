UPDATE configs
SET
    dedup_key = 'v1'
    || '|protocol=' || LENGTH(protocol) || ':' || protocol
    || '|address=' || LENGTH(address) || ':' || address
    || '|port=' || LENGTH(port::TEXT) || ':' || port
    || CASE
        WHEN username IS NULL THEN '|username=-'
        ELSE '|username=' || LENGTH(username) || ':' || username
    END
    || CASE
        WHEN uuid IS NULL THEN '|uuid=-'
        ELSE '|uuid=' || LENGTH(uuid) || ':' || uuid
    END
    || CASE
        WHEN password IS NULL THEN '|password=-'
        ELSE '|password=' || LENGTH(password) || ':' || password
    END
    || CASE
        WHEN method IS NULL THEN '|method=-'
        ELSE '|method=' || LENGTH(method) || ':' || method
    END
    || '|network=' || LENGTH(network) || ':' || network
    || CASE
        WHEN tls IS NULL THEN '|tls=-'
        ELSE '|tls=' || LENGTH(tls) || ':' || tls
    END
    || CASE
        WHEN sni IS NULL THEN '|sni=-'
        ELSE '|sni=' || LENGTH(sni) || ':' || sni
    END
    || CASE
        WHEN host IS NULL THEN '|host=-'
        ELSE '|host=' || LENGTH(host) || ':' || host
    END
    || CASE
        WHEN path IS NULL THEN '|path=-'
        ELSE '|path=' || LENGTH(path) || ':' || path
    END;
