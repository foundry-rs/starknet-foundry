# Normalizes the Sierra debug info before comparison so artifacts generated in
# different environments can still be compared deterministically. We sort fields
# whose order is not meaningful and normalize source paths inside annotations by
# trimming machine-specific prefixes while preserving the stable suffix.

def normalize_path:
  split("/") as $parts
  # Trim the machine-specific absolute path prefix and keep the stable suffix.
  | if ($parts | index("registry")) != null then
      ($parts | (index("registry")) as $i | .[$i:] | join("/"))
    elif ($parts | index("src")) != null then
      ($parts | (index("src")) as $i | .[$i:] | join("/"))
    else
      .
    end;

def normalize_annotation_payload:
  if has("statements_code_locations") then
    .statements_code_locations |= with_entries(
      .value |= map(
        if (type == "array") and (length >= 1) and (.[0] | type) == "string" then
          .[0] |= normalize_path
        else
          .
        end
      )
    )
  else
    .
  end;

def normalize_debug_info:
  # Sort name mappings so equivalent debug info compares the same even if
  # the compiler emits these arrays in a different order.
  (if has("type_names") then .type_names |= sort_by(.[0]) else . end)
  | (if has("libfunc_names") then .libfunc_names |= sort_by(.[0]) else . end)
  | (if has("user_func_names") then .user_func_names |= sort_by(.[0]) else . end)
  | (if has("annotations")
     then .annotations |= with_entries(.value |= normalize_annotation_payload)
     else .
     end);

if has("sierra_program_debug_info")
then .sierra_program_debug_info |= normalize_debug_info
else .
end
