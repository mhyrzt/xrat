#!/bin/sh
set -eu
cd "$(dirname "$0")"
operation="$1"
chain="${2:-}"
if [ -f fail-query ] && { [ "$operation" = -C ] || [ "$operation" = -S ]; }; then
    echo 'permission denied' >&2
    exit 4
fi
case "$operation" in
    -C)
        if [ ! -f "$4" ]; then exit 2; fi
        test -f "$4.attached"
        exit $? ;;
    -S)
        for name in XRAT_KS XRAT_KS_STG; do
            if [ -f "$name" ]; then printf -- '-N %s\n' "$name"; fi
        done
        exit 0 ;;
esac
if [ -f "fail${operation}" ]; then
    echo "injected $operation failure" >&2
    exit 1
fi
case "$operation" in
    -N) test ! -f "$chain"; touch "$chain" ;;
    -A)
        test -f "$chain"
        for argument in "$@"; do
            if [ "$argument" = DROP ]; then touch "$chain.drop"; fi
        done ;;
    -I) test -f "$5"; touch "$5.attached" ;;
    -D) test -f "$4.attached"; rm "$4.attached" ;;
    -F) test -f "$chain"; rm -f "$chain.drop" ;;
    -X) test ! -f "$chain.attached"; rm "$chain" ;;
    -E)
        test ! -f "$3"
        mv "$chain" "$3"
        for suffix in drop attached; do
            if [ -f "$chain.$suffix" ]; then mv "$chain.$suffix" "$3.$suffix"; fi
        done ;;
    *) exit 2 ;;
esac
if [ -f guard ] && ! {
    { [ -f XRAT_KS.attached ] && [ -f XRAT_KS.drop ]; } ||
    { [ -f XRAT_KS_STG.attached ] && [ -f XRAT_KS_STG.drop ]; }
}; then
    touch leaked
fi
