# ============================================================================
# grass.ai top-level Makefile
#
# Copyright (c) 2026 Abhijna Micro (OPC) PVT LTD
# Author: Abhijna Micro Team <team@abhijnamicro.in>
#
# Supported targets:
#   make build [platform=alinux|android]
#   make test  [platform=alinux|android] [filter=<name>]
#   make check / lint / fmt / clippy
#   make example-examples (build all user examples)
#   make ci  (full CI: fmt -> clippy -> build -> test)
#
# Platform defaults:
#   platform = alinux   (host Linux x86_64/aarch64)
#   platform = android  (cargo-ndk Android targets: arm64-v8a, armeabi-v7a, x86_64)
# ============================================================================

SHELL := /bin/bash

DEFAULT_PLATFORM := alinux
PLATFORM         ?= $(DEFAULT_PLATFORM)

# ---- Release / debug -------------------------------------------------------
BUILD_MODE ?= release
ifeq ($(BUILD_MODE),release)
CARGO_BUILD_FLAGS := --release
CARGO_TEST_FLAGS  := --release
else
CARGO_BUILD_FLAGS :=
CARGO_TEST_FLAGS  :=
endif

# ---- Rust toolchain --------------------------------------------------------
CARGO   ?= cargo
RUSTUP  ?= rustup

# ---- Android targets (cargo-ndk) -------------------------------------------
ANDROID_ARCHES      := arm64-v8a armeabi-v7a x86_64
NDK_HOME           ?= $(HOME)/Android/Sdk/ndk/$(shell ls -t $(HOME)/Android/Sdk/ndk 2>/dev/null | head -1)
ifeq ($(strip $(NDK_HOME)),$(HOME)/Android/Sdk/ndk/)
NDK_HOME := /opt/android-ndk
endif

# ---- CI integration --------------------------------------------------------
CIRCLE_BRANCH      ?= main
JENKINS_WORKSPACE  ?= $(WORKSPACE)

# ---- Cross/NDK helpers -----------------------------------------------------
ifeq ($(PLATFORM),android)
RUST_TARGETS := aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
NDK_CMD := cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64
else
RUST_TARGETS := x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
NDK_CMD :=
endif

# ---- Colors ----------------------------------------------------------------
ifeq ($(NO_COLOR),)
GREEN := \033[0;32m
YELLOW:= \033[0;33m
RED   := \033[0;31m
RESET := \033[0m
endif

info = @printf "$(GREEN)==> $(1)$(RESET)\n"
warn = @printf "$(YELLOW)!!> $(1)$(RESET)\n"

# ============================================================================
# Top-level convenience targets
# ============================================================================

.PHONY: help all build test lint check fmt clippy ci \
        clean purge \
        android-build android-test \
        alinux-build alinux-test \
        example-examples run-examples \
        install-tools setup-android setup-ndk \
        print-platform

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-22s\033[0m %s\n", $$1, $$2}'

all: build test ## Build + test (alias)

# ============================================================================
# Build targets
# ============================================================================

build: ## Build workspace for the selected platform
	@$(call info,Building workspace for platform=$(PLATFORM) mode=$(BUILD_MODE))
	@if [ "$(PLATFORM)" = "android" ]; then \
		$(MAKE) android-build; \
	else \
		$(MAKE) alinux-build; \
	fi

alinux-build: ## Build for host Linux
	@$(call info,Building for host Linux)
	$(CARGO) build --workspace $(CARGO_BUILD_FLAGS)

android-build: ## Build all crates + examples for Android via cargo-ndk
	@$(call info,Building for Android)
	@$(MAKE) check-ndk
	@for target in $(ANDROID_ARCHES); do \
		$(call info,Android target: $$target); \
		$(CARGO) ndk -t $$target build --workspace $(CARGO_BUILD_FLAGS); \
	done

# ============================================================================
# Test targets
# ============================================================================

test: ## Run all tests (unit + integration) for the selected platform
	@$(call info,Testing workspace for platform=$(PLATFORM) mode=$(BUILD_MODE))
	@if [ "$(PLATFORM)" = "android" ]; then \
		$(MAKE) android-test; \
	else \
		$(MAKE) alinux-test; \
	fi

alinux-test: ## Run all tests on host Linux
	@$(call info,Running all tests on host Linux)
	$(CARGO) test --workspace $(CARGO_TEST_FLAGS)
	@$(call info,Running per-module unit tests + integration tests)
	$(CARGO) test --test integration_tests $(CARGO_TEST_FLAGS) -- --nocapture
	$(CARGO) test --test module_unit_tests $(CARGO_TEST_FLAGS)

android-test: ## Run host-side tests (device tests need NDK/emulator)
	@$(call warn,Full device test execution requires an emulator; running host-side unit tests)
	$(CARGO) test $(CARGO_TEST_FLAGS)

test-one: ## Run a single test: make test-one filter=<name>
	@$(call info,Running tests matching '$(filter)')
	$(CARGO) test --workspace $(CARGO_TEST_FLAGS) $(filter) -- --nocapture

# ============================================================================
# Code quality targets
# ============================================================================

lint: clippy ## Alias for clippy

check: ## cargo check (fast compile check, no codegen)
	@$(call info,cargo check for platform=$(PLATFORM))
	@if [ "$(PLATFORM)" = "android" ]; then \
		$(CARGO) ndk -t arm64-v8a check --workspace; \
	else \
		$(CARGO) check --workspace --all-targets; \
	fi

fmt: ## Format all code (check mode)
	@$(call info,Checking formatting)
	$(CARGO) fmt --all -- --check

fmt-fix: ## Apply formatting fixes
	@$(call info,Applying formatting)
	$(CARGO) fmt --all

clippy: ## Run clippy lints
	@$(call info,Running clippy)
	$(CARGO) clippy --workspace --all-targets -- -W clippy::all

clippy-strict: ## Run clippy with warnings denied (ideal CI gate)
	@$(call info,Running clippy (strict))
	$(CARGO) clippy --workspace --all-targets -- -D warnings

# ============================================================================
# Example targets
# ============================================================================

EXAMPLES := modbus_power_meter iot_sensor_kafka_influx \
            people_count_display city_beat_count

example-examples: ## Build all user-supplied examples (examples.new.txt)
	@printf "$(GREEN)==> Building all examples$(RESET)\n"
	@for name in $(EXAMPLES); do \
		printf "$(GREEN)==> Building example: $$name$(RESET)\n"; \
		$(CARGO) build --example $$name $(CARGO_BUILD_FLAGS) || { \
			printf "$(RED)FAILED: example $$name$(RESET)\n"; \
			exit 1; \
		}; \
	done
	@printf "$(GREEN)All examples built OK$(RESET)\n"

run-examples: ## Run all examples (loop) demonstrating the framework
	@$(call info,Running examples sequentially)
	@for name in $(EXAMPLES); do \
		$(call info,Running: cargo run --example $$name); \
		$(CARGO) run --example $$name || true; \
	done

# ============================================================================
# Tooling / prerequisites
# ============================================================================

install-tools: ## Install required Rust + NDK tooling
	@$(call info,Installing tooling)
	@rustup component add rustfmt clippy 2>/dev/null || true
	@cargo install cargo-ndk --locked 2>/dev/null || true
	@for target in $(RUST_TARGETS); do \
		rustup target add $$target 2>/dev/null || true; \
	done

setup-android: setup-ndk ## Configure Android build prerequisites

setup-ndk: ## Install Android NDK rust targets via rustup
	@$(call info,Adding Android rust targets)
	@rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
	@$(call info,Ensuring cargo-ndk is installed)
	@which cargo-ndk || cargo install cargo-ndk --locked

check-ndk:
	@if ! command -v cargo-ndk >/dev/null 2>&1; then \
		printf "$(RED)cargo-ndk is required for Android builds. Run: make setup-android$(RESET)\n"; \
		exit 1; \
	fi
	@if [ ! -d "$(NDK_HOME)" ]; then \
		printf "$(YELLOW)NDK not found at $(NDK_HOME); set NDK_HOME=<path to android ndk>$(RESET)\n"; \
	fi

# ============================================================================
# CI targets (used by CircleCI + Jenkins)
# ============================================================================

ci: fmt clippy build test example-examples ## Full CI pipeline
	@printf "$(GREEN)CI pipeline completed OK (platform=$(PLATFORM))$(RESET)\n"

circleci-command: ## Invoked from .circleci/config.yml
	@$(MAKE) fmt
	@$(MAKE) clippy
	@$(MAKE) build
	@$(MAKE) test
	@$(MAKE) example-examples

jenkins-command: ## Invoked from Jenkinsfile
	@$(call info,Jenkins workspace: $(JENKINS_WORKSPACE))
	@$(MAKE) fmt
	@$(MAKE) clippy
	@$(MAKE) build
	@$(MAKE) test
	@$(MAKE) example-examples

# ============================================================================
# Cleanup
# ============================================================================

clean: ## Remove build artifacts
	@$(call info,Cleaning)
	$(CARGO) clean

purge: clean ## Full clean incl. registry cache
	@rm -rf target && $(CARGO) clean

print-platform: ## Print resolved build parameters
	@printf "PLATFORM = $(PLATFORM)\n"
	@printf "BUILD_MODE = $(BUILD_MODE)\n"
	@printf "CARGO = $(CARGO)\n"
	@printf "NDK_HOME = $(NDK_HOME)\n"
	@printf "ANDROID_ARCHES = $(ANDROID_ARCHES)\n"
	@printf "CIRCLE_BRANCH = $(CIRCLE_BRANCH)\n"