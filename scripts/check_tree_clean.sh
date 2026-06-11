#!/usr/bin/env bash
set -euo pipefail

DENYLIST_HASHES=(
  "bccc86317c281a19e56f3f941077ea61b3fb37d5bd527c518b92b188c4461251"
  "b951327c37d8a1ed4d5ed18908681f6aca718ecdfe174aa39cf2237547f92fe7"
  "0bbe5bc4573bcf1caa8ba13b58efe7c468bb397ddfe2f5a83f9c76ba92cec5b4"
  "4242c7f6d3a7e888936effebac1bf59c25837e071968fddfd885dbd6feaba149"
  "8f460fc4d894021da121ecbf60c078b0a1cea232a2044fb2baa81d1af3e12337"
  "6f3dfaf03456fbf15f47e5374445a2709ab8149a40458d63842437838cc0d03b"
  "e026f56c08dcdea441a2aa620ab410ce91162062609b2430f5a13bc9b7cf93f1"
  "2b6873defee36f1396d6a35f01c1e229cddc1db58fb19aea9bd3f7912eac5b9d"
  "9ddde294843122274ad7332e61e57115775231c42d62e0fa4cca1dd30b90a34c"
  "3672100d780398d595025e75fc0fa20be4d7656c674b9900ce60959bf1c3a28b"
  "9bcb07239d88ef7c87f733534fdf5fee50fb213e115cebb1cd8f4b519c2186f1"
  "359cec2f69d14ac180b92598e4463e10c2d8f22ff5388bcf19b0411f86fbedae"
  "24f818c99965183008b728f1585e20a3923e08ebd68dc577245b10d01752499f"
  "7ae45ad102eab3b6d7e7896acd08c427a9b25b346470d7bc6507b6481575d519"
  "053150b640a7ce75eff69d1a22cae7f0f94ad64ce9a855db544dda0929316519"
  "32824c984905bb02bc7ffcef96a77addd1f1602cff71a11fbbfdd7f53ee026bb"
  "385bfa2b4522184962dd7592b5877648b51c6c0c0975d8ff0b388f552702fb29"
  "ac7148ee4d01e3aed55f1fe8177f2ac48494689fa83a15a74dfa03243799921c"
)

if [ "${#DENYLIST_HASHES[@]}" -eq 0 ]; then
  echo "No denylist hashes configured" >&2
  exit 2
fi

fail=0

hash_norm() {
  printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
}

is_denied_hash() {
  local hash="$1"
  local denied
  for denied in "${DENYLIST_HASHES[@]}"; do
    [ "$hash" = "$denied" ] && return 0
  done
  return 1
}

while IFS= read -r tracked || [ -n "$tracked" ]; do
  norm="$(printf '%s' "$tracked" | tr '[:upper:]' '[:lower:]' | sed 's:/*$::')"
  probe="$norm"
  while [ -n "$probe" ] && [ "$probe" != "." ]; do
    if is_denied_hash "$(hash_norm "$probe")"; then
      echo "DENYLISTED PATH TRACKED: $tracked" >&2
      fail=1
      break
    fi
    case "$probe" in
      */*) probe="${probe%/*}" ;;
      *) break ;;
    esac
  done
done < <(git ls-files)

exit "$fail"
