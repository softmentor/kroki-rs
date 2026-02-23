# Professional Unified Makefile for Kroki-rs
# Standardized targets: setup, devrun (develop), cirun (ci-verify), ghrun, teardown
# Modular includes under src-scripts/develop (vars, native, container, repro)

include src-scripts/develop/vars/vars.mk
include src-scripts/develop/native/native.mk
include src-scripts/develop/container/container.mk
include src-scripts/develop/repro/repro.mk

.DEFAULT_GOAL := help
