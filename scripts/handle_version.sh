#!/bin/bash

get_version() {
  local overridden_version=$1
  local plugin_path=$2
  local default_version
  default_version=$(grep "^version =" "$plugin_path" | cut -d '"' -f 2)

  if [ -z "$overridden_version" ]; then
    echo "$default_version"
  else
    echo "$overridden_version"
  fi
}

update_version_in_file() {
  local file=$1
  local version=$2

  if [ -n "$version" ]; then
    sed -i.bak "/\[package\]/,/version =/ s/version = \".*/version = \"$version\"/" "$file"
    rm "$file.bak" 2> /dev/null
  fi
}

export -f get_version
export -f update_version_in_file
