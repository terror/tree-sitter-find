set dotenv-load

default:
	just --list

alias f := fmt
alias r := run
alias t := test

all: build test clippy fmt-check

[group: 'misc']
build:
  cargo build

[group: 'check']
check:
 cargo check

[group: 'check']
ci: test clippy forbid
  cargo +nightly fmt --all -- --check
  cargo update --locked --package tsql

[group: 'check']
clippy:
  cargo clippy --workspace --all-targets -- --deny warnings

[group: 'format']
fmt:
  cargo +nightly fmt

[group: 'format']
fmt-check:
  cargo +nightly fmt --all -- --check

[group: 'check']
forbid:
  ./bin/forbid

[group: 'dev']
install-dev-deps:
  rustup install nightly
  rustup update nightly
  cargo install cargo-watch

[group: 'dev']
run *args:
  cargo run -- --{{args}}

[group: 'test']
test:
  cargo test

[group: 'release']
update-changelog:
  echo >> CHANGELOG.md
  git log --pretty='format:- %s' >> CHANGELOG.md

[group: 'dev']
watch +COMMAND='test':
  cargo watch --clear --exec "{{COMMAND}}"
