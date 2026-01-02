# PolyShark Demo Script (3-4 minutes)

## Overview

This demo shows PolyShark's complete permission lifecycle with MetaMask ERC-7715.

---

## Scene 1: Connect & Configure (0:00-1:00)

**Action:** Open dashboard, configure permission

1. Show dashboard at `dashboard/index.html`
2. Adjust daily limit slider (e.g., $20/day)
3. Select duration (30 days)
4. Toggle dry-run mode ON for demo safety
5. Click "View JSON Config" to show the ERC-7715 permission object

**Talking Point:**
> "PolyShark lets users configure exactly how much the agent can spend. The JSON you see is the actual ERC-7715 permission request."

---

## Scene 2: Grant Permission (1:00-1:30)

**Action:** Connect and grant permission

1. Click "Connect MetaMask"
2. Show MetaMask popup confirming connection
3. Click "Grant Permission"
4. Show the permission confirmation dialog

**Talking Point:**
> "Once granted, this permission is cryptographically enforced. The agent cannot exceed the daily limit—it's not just a promise, it's on-chain enforcement."

---

## Scene 3: Autonomous Trading (1:30-2:30)

**Action:** Watch the agent trade

1. Agent status changes to "RUNNING"
2. Show Envio health indicator (low latency, connected)
3. Watch trades appear in the list
4. Observe:
   - Allowance bar decreasing
   - Strategy mode changing as budget depletes
   - Trade PnL updating

**Talking Point:**
> "Notice there are no wallet popups. The agent is trading autonomously within the permission bounds. The strategy adapts—when allowance is low, it switches to conservative mode, only taking high-edge trades."

---

## Scene 4: Revocation (2:30-3:00)

**Action:** Revoke the permission

1. Click "Revoke Permission"
2. Confirm in dialog
3. Agent status immediately changes to "IDLE"
4. No more trades execute

**Talking Point:**
> "Users maintain full control. Revocation is instant, and the agent respects it immediately. This is the power of ERC-7715—trust-minimized automation."

---

## Scene 5: Summary (3:00-3:30)

**Action:** Return to dashboard overview

Show key metrics:
- Total trades executed
- Win rate
- Net PnL
- Permission lifecycle complete

**Talking Point:**
> "PolyShark demonstrates how ERC-7715 Advanced Permissions enable safe, autonomous agents. Combined with Envio's low-latency data, we get real-time trading without sacrificing user control."

---

## Key Messages to Emphasize

1. **Permission Enforcement** — Not trust-based, cryptographically enforced
2. **Zero Popups** — Better UX after initial grant
3. **Envio Integration** — Low-latency, reliable data source
4. **Adaptive Safety** — Strategy modes based on remaining allowance
5. **User Control** — Instant revocation, always in control
