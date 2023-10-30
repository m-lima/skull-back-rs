#!/usr/bin/env bash

function usage {
  local base=`basename "${0}"`

  echo "USAGE"
  echo "  ${base} COMMAND"
  echo
  echo "COMMANDS"
  echo "  lock    Creates a .sqlx directory based off of store/db/db.sqlite"
  echo "  unlock  Creates or resets a database at store/db/db.sqlite"
  echo "  help    Prints this help message"
}

function error {
  echo -n "${1}" >&2
  if [ "${2}" ]; then
    echo " ${2}" >&2
  else
    echo >&2
  fi

  echo >&2
  usage >&2
  exit 1
}

function unlock {
  local base="${1}"

  cd "${base}"
  mkdir -p "${base}/db"
  sqlx database reset -D "sqlite://${base}/db/db.sqlite"
  echo -n "DATABASE_URL=sqlite://store/db/db.sqlite" > "${base}/.env"
}

function lock {
  local base="${1}"

  if [ ! -d "${base}/db" ] || [ ! -f "${base}/db/db.sqlite" ]; then
    echo "Database does not exist at ${base}/db/db.sqlite"
    echo -n "Create? [Y/n] "
    read input
    case "${input}" in
      [Nn]) exit 1 ;;
      *) unlock "${base}" || exit 1 ;;
    esac
  fi

  cd "${base}"
  cargo sqlx prepare -D "sqlite://${base}/db/db.sqlite"
  rm "${base}/.env"
}

base=$(realpath "$(dirname "${0}")")

case "${1}" in
  "lock") lock "${base}" ;;
  "unlock") unlock "${base}" ;;
  *) error "Unknown parameter" ;;
esac
