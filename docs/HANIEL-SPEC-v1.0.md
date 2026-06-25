# HANIEL — AIEONYX Sovereign Browser and Rendering Engine
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0

## Mission

HANIEL is the sovereign browser and universal rendering engine of the AIEONYX
civilization stack. It replaces WebKitGTK in Onyxia and powers every rendering
surface in the AIEONYX ecosystem — browser, desktop, mobile, kiosk, media player,
AI runtime, and document renderer.

Named after Haniel Lepiten — first son of Edison Lepiten, founder of AIEONYX.
Every pixel HANIEL renders carries that name.

## Architecture — 8 Sovereign Modules

| Module          | Crate              | Function                                      |
|-----------------|--------------------|-----------------------------------------------|
| HERALD          | haniel_herald      | Network, protocol, STS threat gate, ARPi      |
| PRISM           | haniel_prism       | Layout engine — rod/cone dual pass            |
| CANVAS          | haniel_canvas      | Rasterizer and GPU renderer                   |
| ECHO            | haniel_echo        | Script runtime — AxonScript + WASM            |
| HANIEL-ONYX     | haniel_onyx        | On-device AI compute layer                    |
| VAULT           | haniel_vault       | Memory manager and sovereign cache            |
| SENTINEL        | haniel_sentinel    | seL4 formally isolated renderer               |
| LUMEN           | haniel_lumen       | Sovereign media engine — video, audio, stream |

## AXON Language Foundation

HANIEL is written 100% in AXON (via Rust host) from day one.
AXON P56–P63 complete. All sovereign libraries available:

- axon_net        → HERALD network layer
- axon_crypto     → all security layers
- axon_gpu        → CANVAS GPU rendering
- axon_media      → LUMEN sovereign codecs
- axon_layout     → PRISM layout engine
- axon_wasm       → ECHO web compatibility
- axon_font       → text rendering
- axon_ai_runtime → HANIEL-ONYX sovereign AI
- axon_sel4       → SENTINEL formal isolation

## Build Phases

| Phase | Module   | Deliverable                          | Stage |
|-------|----------|--------------------------------------|-------|
| HE-1  | ALL      | Workspace skeleton + CI              | 1     |
| HE-2  | VAULT    | Memory + cache core                  | 1     |
| HE-3  | HERALD   | Network + STS + AWP                  | 1     |
| HE-4  | PRISM    | .axbw parser + rod pass              | 1     |
| HE-5  | PRISM    | Cone pass + flexbox + HTML           | 1     |
| HE-6  | CANVAS   | Software rasterizer (first pixel)    | 1     |
| HE-7  | CANVAS   | GPU path (wgpu/Vulkan)               | 2     |
| HE-8  | VAULT    | Font engine + text rendering         | 2     |
| HE-9  | ECHO     | Script runtime + capability DOM      | 2     |
| HE-10 | LUMEN    | Sovereign media — video + audio      | 2     |
| HE-11 | ONYX     | On-device AI compute layer           | 2     |
| HE-12 | SENTINEL | seL4 process isolation               | 2     |
| HE-13 | ALL      | Full AWP engine integration          | 3     |
| HE-14 | ALL      | Open web + YouTube compatibility     | 3     |
| HE-15 | ALL      | WebKitGTK removal — full sovereignty | 3     |

## Sovereign Innovations

- TERM-049: Threat-First Rendering Pipeline
- TERM-050: Rod-Cone Progressive Rendering
- TERM-051: Sovereign Render Budget (SRB)
- TERM-052: Capability-Gated DOM
- TERM-053: AWP-Native Engine Path
- TERM-054: Formally Isolated Renderer
- TERM-055: AI-Aware Layout Engine
- TERM-056: Zero-GC Render Pipeline
- TERM-057: On-Device AI Render Target
- TERM-058: Biological Renderer Architecture

## Naming

HANIEL is named after Haniel Lepiten, first son of Edison Lepiten.
The HANIEL browser engine is the first sovereign rendering engine
built for a digital civilization rather than a corporation.

Copyright (c) 2026 Edison Lepiten / AIEONYX
