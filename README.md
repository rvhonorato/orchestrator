# orchestrator

![GitHub License](https://img.shields.io/github/license/rvhonorato/orchestrator)
![GitHub Release](https://img.shields.io/github/v/release/rvhonorato/orchestrator)
[![ci](https://github.com/rvhonorato/orchestrator/actions/workflows/ci.yml/badge.svg)](https://github.com/rvhonorato/orchestrator/actions/workflows/ci.yml)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/7f2a8816886645d28cbaac0fead038f9)](https://app.codacy.com/gh/rvhonorato/orchestrator/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)

## Overview

This is a central component [WeNMR](https://wenmr.science.uu.nl), a worldwide
e-Infrastructure for NMR and structural biology - operated by
the [BonvinLab](https://bonvinlab.org) at the [Utrecht University](https://uu.nl).
It is closely coupled with [`jobd`](https://github.com/rvhonorato/jobd),
with more destinations to be added in the future such as:

- [DIRAC interware](https://dirac.readthedocs.io/en/latest/index.html)
- Educational cloud services
- SLURM

This is an asynchronous job orchestration system written in Rust that
manages and distributes computational jobs across research software
apps. Its a reactive middleware layer between the backend and various
computing resources, implementing quota-based load balancing.

```mermaid
flowchart LR
    B([User]) --> C[Web App]
    C[Web App] <--> Y[(Database)]
    C[Web App] --> X{{Orchestrator}}
    X -->|jobd| D[[prodigy]]
    X -->|jobd| E[[disvis]]
    X -->|jobd| G[[other_service]]
    E -->|slurml| H[local HPC]
```

## Implementation

🚧 soon 🚧

## Docs

🚧 soon 🚧

## Contact

If you think this project would be useful for your use case or would like to suggest something, please reach out either via issue here or via email. (:
