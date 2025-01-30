# orchestrator

![GitHub License](https://img.shields.io/github/license/rvhonorato/orchestrator)
![GitHub Release](https://img.shields.io/github/v/release/rvhonorato/orchestrator)
[![ci](https://github.com/rvhonorato/orchestrator/actions/workflows/ci.yml/badge.svg)](https://github.com/rvhonorato/orchestrator/actions/workflows/ci.yml)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/7f2a8816886645d28cbaac0fead038f9)](https://app.codacy.com/gh/rvhonorato/orchestrator/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)


🚧 work in progress 🚧

## Overview

This project is n asynchronous job orchestration system written in Rust that
manages and distributes computational jobs across services
(research software apps). It acts as an intelligent middleware layer between
the backend and various computing resources, implementing quota-based load
balancing and resource management.

The orchestrator receives job payloads from the (WeNMR) backend, tracks
per-user and per-service quotas in real-time, and makes intelligent routing
decisions based on current resource utilization. It either forwards jobs to
appropriate destinations or queues them when quota limits are reached.

Built with Rust's async runtime for high performance and reliability,
this project is currently in alpha stage and designed to be extensible
for supporting additional computing resources and service integrations in
the future.
