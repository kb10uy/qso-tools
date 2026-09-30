#!/bin/bash

if (( $# < 1 )); then
  echo "Specify template file" >&2
  exit 1
fi
if [[ ! -f "$1" ]]; then
  echo "Template file not found" >&2
  exit 1
fi

TEMPLATE_DOCUMENT="$1"
shift 1

WORKDIR=$(mktemp -d)
cp "${TEMPLATE_DOCUMENT}" "${WORKDIR}/template.typ"

for source_file in "$@"; do
  if [[ ! -f "${source_file}" ]]; then
    echo "Source ${source_file} not found"
    continue
  fi
  echo "Processing ${source_file}"
  source_filename=$(basename "${source_file}")

  cp "${source_file}" "${WORKDIR}/${source_filename}"
  typst compile \
    --input "data_json=${source_filename}" \
    "${WORKDIR}/template.typ" \
    "${WORKDIR}/${source_filename}.pdf"
  cp "${WORKDIR}/${source_filename}.pdf" ./
done

rm -rf "${WORKDIR}"
