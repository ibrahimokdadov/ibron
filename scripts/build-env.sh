#!/usr/bin/env bash
# Source this file (not execute) before building ibron on Windows/bash.
# Puts Strawberry Perl portable on PATH so openssl-sys (vendored) can build.
#   source scripts/build-env.sh

export PATH="/c/Users/ibrah/tools/strawberry-perl/perl/bin:/c/Users/ibrah/tools/strawberry-perl/c/bin:$PATH"
