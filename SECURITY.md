# Security Policy

this is currently a learning and experimental project.

nosqlite is not production ready and should not be used for storing sensitive or important data.

things can break.
data corruption can happen.
disk format can change anytime.


## reporting security issues

if you find any serious issue or vulnerability feel free to open an issue or contact privately first before public disclosure.

include:
- what happened
- how to reproduce
- affected version/commit
- possible impact


## scope

currently there is:
- no authentication
- no encryption
- no network layer
- no sandboxing

so security hardening is not main focus right now.

current focus is learning storage engine internals and database architecture.


## safe usage recommendation

do not use this project:
- in production
- for sensitive data
- for anything important
- as a trusted secure storage system

always keep backups of database files while testing.
