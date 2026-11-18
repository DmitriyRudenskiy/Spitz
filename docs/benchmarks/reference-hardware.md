# Reference hardware

The single machine named by [ADR-0008](../../docs/adr/0008-reference-hardware-one-named-machine.md).
Every "on the reference hardware" in the specs means this machine. Fill it in before the first
budget run; when the machine changes, this file and every budget number change in the same change.

| Field | Value |
|---|---|
| Machine (vendor/model or "self-built") | _TBD_ |
| OS + kernel (Linux first-class, ADR-0005) | _TBD_ |
| CPU (exact model) | _TBD_ |
| Cores / threads | _TBD_ |
| RAM | _TBD_ |
| Storage (model class, NVMe/SATA) | _TBD_ |
| Runner tag for `budgets.yml` | `reference` |

Tolerance band between runs: ±20% (ADR-0008). Results outside the band re-run once before anyone
believes or fixes them.
