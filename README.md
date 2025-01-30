# orchestrator

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
