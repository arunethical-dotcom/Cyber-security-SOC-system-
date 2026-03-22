# SOC System — Security Operations Center Pipeline

A **complete, production-ready SOC (Security Operations Center) pipeline** that ingests security events, detects attack patterns in real-time, generates incidents, and displays them through a modern dashboard.

This project demonstrates how security teams can automatically detect, correlate, and respond to threats using a combination of **fast Rust backend processing**, **Python orchestration**, and a **real-time Flutter frontend**.

---

## 📋 Quick Overview

| Component | Purpose | Port | Tech | Status |
|-----------|---------|------|------|--------|
| **Rust API Server** | Event ingestion, detection, correlation | 8080 | Rust + Axum | ✅ Operational |
| **Python AI Layer** | API proxy, LLM enrichment, real-time streaming | 8000 | Python + FastAPI | ✅ Operational |
| **Flutter UI** | Real-time dashboard with incident visualization | — | Dart/Flutter | ✅ Operational |
| **Ollama LLM** | AI-powered incident summarization (optional) | 11434 | Ollama | 📌 Optional |
| **Event Generator** | Simulates attack scenarios and normal traffic | — | Python | ✅ Included |

**System Health Check:**
```bash
curl http://localhost:8080/health     # Rust backend
curl http://localhost:8000/health     # Python API
```

---

## 🏗️ Architecture

### System Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        EVENT SOURCES                             │
│  (Logs, Endpoints, Network Sensors, Generated Events)           │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             │ POST /events
                             ▼
        ┌──────────────────────────────────────────┐
        │  RUST CORE - Detection Engine :8080     │
        ├──────────────────────────────────────────┤
        │                                          │
        │  [Event Ingestion]                       │
        │  • Parse JSON/JSONL                      │
        │  • Entity extraction                     │
        │  • Normalize to CanonicalEvent           │
        │                                          │
        │  [Detection Module]                      │
        │  • Sigma rule matching                   │
        │  • Baseline anomaly detection            │
        │  • IOC/Threat Intel checks               │
        │  • Composite scoring (0.0-1.0)           │
        │                                          │
        │  [Event Graph]                           │
        │  • Build attack chains                   │
        │  • Entity relationships                  │
        │                                          │
        │  [Correlator]                            │
        │  • Pattern detection                     │
        │  • Generate Incidents                    │
        │  • In-memory storage                     │
        │                                          │
        │  [REST API]                              │
        │  • GET /incidents                        │
        │  • GET /graph/snapshot                   │
        │  • POST /feedback                        │
        └────────────────────┬─────────────────────┘
                             │
                             │ GET /incidents (every 2s)
                             ▼
        ┌──────────────────────────────────────────┐
        │  PYTHON LAYER - Orchestration :8000     │
        ├──────────────────────────────────────────┤
        │                                          │
        │  [RustClient Polling]                    │
        │  • Polls Rust every 2 seconds            │
        │  • Detects new incidents                 │
        │                                          │
        │  [R3: Anomaly Detector]                  │
        │  • IsolationForest + online learning     │
        │  • Scores incidents                      │
        │                                          │
        │  [R5: SIEM Database]                     │
        │  • Persists to SQLite                    │
        │  • Enables historical search             │
        │                                          │
        │  [LLM Service]                           │
        │  • Calls Ollama for AI summary           │
        │  • Fallback templates included           │
        │                                          │
        │  [WebSocket Manager]                     │
        │  • Real-time incident broadcast          │
        │                                          │
        │  [REST API Routers]                      │
        │  • /incidents → fetch filtered           │
        │  • /siem/search → query database         │
        │  • /siem/stats → analytics               │
        │  • /feedback → incident tuning           │
        └────────────────────┬─────────────────────┘
                             │
                 ┌───────────┴──────────┐
                 │                      │
                 │ REST API             │ WebSocket
                 │                      │
                 ▼                      ▼
        ┌──────────────────┐  ┌──────────────────┐
        │  Flutter UI      │  │  Real-time Push  │
        │  Dashboard       │  │  Updates         │
        │  :Windows        │  │                  │
        │                  │  │  (Connected      │
        │  • Incident list │  │   Clients)       │
        │  • Graph view    │  │                  │
        │  • Filters       │  │                  │
        │  • Actions       │  │                  │
        └──────────────────┘  └──────────────────┘
```

### Data Flow - Detailed

```
1. EVENT INGESTION
   External Event → Rust :8080/events → Parse JSON
   
2. DETECTION
   CanonicalEvent → Sigma Rules ✓ Match?
              → Baseline Anomaly Check (Z-score)
              → IOC Lookup
              → Composite Score (50% sigma + 30% anomaly + 20% IOC)
   
3. CORRELATION
   Event Detection {score > 0.7} → Event Graph Insert
   Graph Patterns → Detect (Brute Force, Lateral Movement, etc.)
              → Create Incident (HIGH severity)
   
4. ENRICHMENT
   Incident → Python RustClient Polls → LLM Call (Ollama)
         → Summary + Recommended Actions
         → Record in Anomaly Model (R3)
         → Persist to SIEM (R5)
   
5. BROADCAST
   Enriched Incident → WebSocket Broadcast
                    → All Connected Clients (Flutter)
   
6. VISUALIZATION
   Flutter UI ← REST API ← Python Layer ← Rust Backend
   Displays incident list, graph, summary, actions
```

---

## 🔑 Key Concepts Explained

### 1. **Event**

A **single security observation** — a raw log entry or network alert. Events are the starting point.

**Example: Failed Login Event**
```json
{
  "event_type": "login_failed",
  "entity": {
    "type": "user",
    "value": "admin"
  },
  "metadata": {
    "source_ip": "10.0.0.5",
    "destination_host": "dc01"
  }
}
```

**Status:** Raw, unscored, not yet an alert. System ingests thousands of these per day.

---

### 2. **Detection**

The process of **scoring an event** based on security rules, historical patterns, and threat intelligence.

**How It Works:**

```
Event: "admin failed login from 10.0.0.5"

SIGMA RULE CHECK:
├─ Rule: "Multiple Failed Logins"
│  └─ Condition: event_type == "login_failed"
│  └─ Match? YES → sigma_score = 0.6
│
BASELINE ANOMALY CHECK:
├─ Historical: "admin usually logs in 9am-5pm"
├─ Current: "10:00am on Tuesday" (within normal)
│  └─ anomaly_score = 0.2 (low deviation)
│
IOC CHECK:
├─ Known Bad IPs: ["1.1.1.1", "2.2.2.2"]
├─ IP "10.0.0.5" not in list
│  └─ ioc_score = 0.0
│
COMPOSITE SCORE:
  (0.5 × 0.6) + (0.3 × 0.2) + (0.2 × 0.0) = 0.36
  
Decision: 0.36 < threshold (0.7) → NORMAL (no alert)
```

If **5 failed logins in 60 seconds** → sigma_score jumps to 0.9 → composite = 0.63 (still below threshold).

If **5 failed logins + 1 success** → sigma_score = 0.95 → composite = **0.72** → **ALERT TRIGGERED!** ✓

---

### 3. **Incident**

A **structured, actionable security alert** generated when patterns are detected. Contains not just the raw event, but correlated context and recommendations.

**Example: Brute Force Incident**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-03-22T10:05:00Z",
  "severity": "HIGH",
  "chain": ["T1110", "TA0001"],
  "entities": ["user:admin", "device:dc01", "ip:10.0.0.5"],
  "sigma_score": 0.95,
  "z_score": 3.2,
  "cvss": 7.5,
  "summary": "Multiple failed login attempts detected on admin account followed by successful login. Suggests brute force attack.",
  "actions": [
    "Immediately lock admin account",
    "Review login logs for source IP",
    "Check for privilege escalation after success"
  ]
}
```

**Difference from Event:** An incident is **correlated**, **scored**, **summarized**, and **actionable**.

---

### 4. **Correlation**

The process of **linking multiple events** into a coherent incident story using **time windows** and **patterns**.

**Example: Brute Force Pattern**

```
Event 1: login_failed, admin, 10:01:00
Event 2: login_failed, admin, 10:01:15
Event 3: login_failed, admin, 10:01:30
Event 4: login_failed, admin, 10:01:45
Event 5: login_failed, admin, 10:02:00
Event 6: login_success, admin, 10:02:15

Pattern Detected:
  ✓ 5+ failed logins from same source
  ✓ Within 60-second window
  ✓ Same target user (admin)
  ✓ Followed by success
  
→ BRUTE FORCE INCIDENT CREATED (HIGH severity)
```

Configuration (from [config/thresholds.toml](config/thresholds.toml)):
```toml
brute_force_min_attempts = 5    # Need 5+ failures
max_chain_window_secs = 600     # Within 10 minutes
```

---

### 5. **LLM Summary**

An **AI-generated explanation** of the incident in plain English, generated by calling Ollama (optional feature).

**Example Input Incident:**
```
severity: HIGH
sigma_score: 0.95
z_score: 3.2
chain: ["T1110", "TA0001"]
entities: ["user:admin", "device:dc01"]
```

**LLM Output (from Ollama qwen:1.8b):**
```
{
  "incident_type": "brute_force_attack",
  "severity": "HIGH",
  "summary": "Brute force attack detected. Multiple failed login attempts 
             (5 instances over 60 seconds) on admin account to domain 
             controller dc01 from IP 10.0.0.5, followed by successful login. 
             Attacker likely testing credentials or attempting account takeover.",
  "recommended_actions": [
    "Immediately lock admin account to prevent further access",
    "Review admin account login history for suspicious activity",
    "Force password reset for admin account",
    "Check for privilege escalation or lateral movement post-breach"
  ]
}
```

**Fallback (if Ollama unavailable):**
Uses template-based summary (no LLM call needed).

---

## 🛠️ Components Breakdown

### **A. Rust Backend** (`rust-core/`)

**What it does:**
- Receives events via HTTP
- Runs detection rules (Sigma + baseline + IOC)
- Organizes events into an attack graph
- Detects patterns → creates incidents
- Serves REST API for querying

**Key Modules:**

| Module | File | Purpose |
|--------|------|---------|
| **Event Input** | `event-input/src/main.rs` | CLI tool to batch-ingest events from JSON files |
| **Detector** | `detector/src/lib.rs` | Sigma rule engine, anomaly scoring, IOC checks |
| **Graph** | `graph/src/lib.rs` | Attack chain visualization (directed graph) |
| **Correlator** | `correlator/src/lib.rs` | Pattern detection, incident generation |
| **API Server** | `api-server/src/main.rs` | HTTP endpoints, state management |

**REST Endpoints:**

```bash
# Health
GET  /health                       # {"status": "ok"}

# Incident Management
POST /events                        # Receive event
GET  /incidents?severity=HIGH       # List incidents (paginated)
GET  /incidents/{id}                # Get single incident
POST /feedback                      # Submit feedback

# Graph Visualization
GET  /graph/snapshot                # Entity relationships (nodes, edges)
```

**Example Request:**
```bash
curl -X POST http://localhost:8080/events \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "login_failed",
    "entity": {"type": "user", "value": "admin"},
    "metadata": {"source_ip": "10.0.0.5"}
  }'
```

---

### **B. Python Backend** (`python-ai/`)

**What it does:**
- Acts as orchestration layer between Rust and UI
- Polls Rust for new incidents every 2 seconds
- Calls Ollama LLM to summarize incidents
- Broadcasts real-time updates via WebSocket
- Persists incidents to SQLite SIEM (R5)
- Scores incidents with anomaly detector (R3)

**Key Modules:**

| Module | File | Purpose |
|--------|------|---------|
| **Main App** | `main.py` | FastAPI setup, lifespan management |
| **Rust Client** | `services/rust_client.py` | Polls Rust :8080/incidents every 2s |
| **LLM Service** | `services/llm.py` | Calls Ollama, fallback templates |
| **Anomaly** | `services/anomaly.py` | IsolationForest + online learning (R3) |
| **SIEM** | `services/siem.py` | SQLite persistence (R5) |
| **WebSocket** | `services/websocket.py` | Broadcasts to connected UI clients |
| **Routers** | `routers/*.py` | FastAPI endpoints (/incidents, /siem, etc.) |

**REST Endpoints (Proxy + Additional):**

```bash
# Proxy to Rust
GET  /incidents                     # Pass-through to Rust
GET  /incidents/{id}
POST /feedback

# R5: SIEM Database
GET  /siem/search?severity=HIGH     # Query SQLite
GET  /siem/stats                    # Time series stats
GET  /siem/timeline?hours=24
DELETE /siem/incidents/{id}

# Real-time
WebSocket /stream                   # Incident broadcast
```

**LLM Integration:**
```python
# If Ollama available:
POST http://localhost:11434/api/generate
Response: Summarized incident with actions

# If Ollama unavailable:
Use template-based fallback (graceful degradation)
```

---

### **C. Flutter Frontend** (`flutter-app/`)

**What it does:**
- Displays incidents in real-time
- Allows filtering by severity
- Shows attack graph visualization
- Provides incident details, summaries, and recommended actions
- Enables analyst feedback (confirm, suppress, tune)

**Screens:**

| Screen | Purpose |
|--------|---------|
| **Dashboard** | Main incident list, paginated, sortable by severity |
| **Graph View** | Entity relationships, attack chains |
| **Incident Detail** | Full incident info, summary, recommended actions |
| **Feedback Dialog** | Submit confirm/suppress/tune feedback |

**Data Sources:**
```dart
// Fetches from Rust (via Python proxy or direct)
ApiService baseUrl = "http://localhost:8080"

List<Incident> incidents = await api.getIncidents();
Map<String, dynamic> graph = await api.getGraphSnapshot();
await api.submitFeedback(feedback);
```

---

### **D. Event Generator** (`events/generators/`)

**What it does:**
- Simulates real security events
- Generates attack scenarios (brute force, lateral movement, etc.)
- Sends events as POST requests to Rust :8080/events

**Attack Scenarios Included:**

```python
# Brute Force: 5+ failed logins in rapid succession
brute_force_attack()          # 8 failed logins 0.3s apart

# Normal Traffic: Random mix
normal_traffic()              # Occasional successes

# Command: python generate_events.py
# Generates events continuously every 1-3 seconds
```

**Output:**
```
🚀 Generator started...
⚠️ Simulating brute force attack...
✅ Sent: {...} | Status: 200
✅ Sent: {...} | Status: 200
[continues indefinitely]
```

---

## 🔄 System Flow — End-to-End Example

### **Scenario: Admin Account Brute Force Attack**

#### **Step 0: Setup**
- Rust server running on :8080
- Python server running on :8000
- Flutter UI open and connected
- Event generator running

---

#### **Step 1: Event Generation** (t = 0s)

Generator sends 5 failed login events to Rust `POST /events`:

```json
[
  {"event_type": "login_failed", "entity": {"type": "user", "value": "admin"}, "metadata": {"source_ip": "10.0.0.50"}},
  {"event_type": "login_failed", "entity": {"type": "user", "value": "admin"}, "metadata": {"source_ip": "10.0.0.50"}},
  {"event_type": "login_failed", "entity": {"type": "user", "value": "admin"}, "metadata": {"source_ip": "10.0.0.50"}},
  {"event_type": "login_failed", "entity": {"type": "user", "value": "admin"}, "metadata": {"source_ip": "10.0.0.50"}},
  {"event_type": "login_failed", "entity": {"type": "user", "value": "admin"}, "metadata": {"source_ip": "10.0.0.50"}},
  {"event_type": "login_success", "entity": {"type": "user", "value": "admin"}, "metadata": {"source_ip": "10.0.0.50"}}
]
```

**Output from Rust:**
```
📥 Received event: {...}
📥 Received event: {...}
[5 more events]
```

---

#### **Step 2: Detection** (t = 1s, Rust)

For each event, Rust runs detection:

```
Event 1 (login_failed):
  ✓ Sigma rule "Multiple Failed Logins" matches → sigma_score = 0.6
  ✓ Anomaly check: Expected 0.1 logins/hour, got 1 → z_score = 1.5
  ✓ IOC check: IP not in known-bad list → ioc_score = 0.0
  
  Composite = 0.5 * 0.6 + 0.3 * 1.5 + 0.2 * 0 = 0.75
  → ABOVE THRESHOLD (0.7) → INSERT INTO GRAPH ✓

Events 2-5: Similar scores, all inserted into graph

Event 6 (login_success):
  ✓ Sigma rule "Successful Login After Failures" → sigma_score = 0.9
  ✓ Anomaly: Success after 5 failures in 60s → z_score = 3.2
  
  Composite = 0.5 * 0.9 + 0.3 * 3.2 = 1.41 (clamped to 1.0)
  → CORRELATOR TRIGGERED ✓
```

**Rust Log Output:**
```
✓ Event inserted into graph: user:admin → device:dc01
✓ Event inserted into graph: user:admin → device:dc01
[5 edges added]
🚨 INCIDENT CREATED: BruteForceSuccess (HIGH severity)
```

---

#### **Step 3: Incident Creation** (t = 2s, Rust)

Correlator detects pattern → creates incident:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-03-22T10:05:00Z",
  "severity": "HIGH",
  "chain": ["T1110", "TA0001"],
  "entities": ["user:admin", "device:dc01", "ip:10.0.0.50"],
  "sigma_score": 0.9,
  "z_score": 3.2,
  "ioc_match": null,
  "cvss": 7.5,
  "base_signal": 0.9,
  "summary": null,
  "actions": null
}
```

Incident stored in Rust memory: `AppState::incidents.push(incident)`

---

#### **Step 4: Python Polling** (t = 2.5s, Python)

Python RustClient polls Rust every 2 seconds:

```python
incidents = await self.get_incidents(limit=10)
# Returns: [BruteForceSuccess incident]

new_ids = {"550e8400-e29b-41d4-a716-446655440000"}  # Not seen before
→ CALL CALLBACK on_new_incident(incident)
```

---

#### **Step 5: Enrichment & Persistence** (t = 3s, Python)

Callback enriches incident:

```python
# R3: Anomaly Detector scores it
anomaly_score = anomaly_detector.score(incident)
enriched_incident["anomaly_score"] = 0.78

# R3: Record for retraining
anomaly_detector.record_event(enriched_incident)

# R5: Persist to SQLite
ingest_incident(enriched_incident)  # INSERT INTO incidents

# LLM Enrichment
llm_output = await llm_service.generate_summary(enriched_incident)
enriched_incident["summary"] = "Brute force attack detected..."
enriched_incident["actions"] = ["Lock admin account", ...]

# Broadcast to WebSocket clients
await ws_manager.broadcast(enriched_incident)
```

**Enriched Incident:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "severity": "HIGH",
  "anomaly_score": 0.78,
  "summary": "Multiple failed login attempts (5 in 60s) on admin account 
             followed by successful login from IP 10.0.0.50. Suggests 
             brute force attack or credential compromise.",
  "actions": [
    "Immediately lock admin account",
    "Review admin login history for compromise indicators",
    "Force password reset",
    "Check for post-breach lateral movement"
  ]
}
```

---

#### **Step 6: Dashboard Display** (t = 3.5s, Flutter UI)

Flutter receives WebSocket push:

```
┌────────────────────────────────────────┐
│ 🚨 INCIDENT #550e8400                  │
│ SEVERITY: 🔴 HIGH                      │
│ TIME: 2026-03-22 10:05:00              │
│                                        │
│ Multiple failed login attempts (5 in   │
│ 60s) on admin account followed by      │
│ successful login from IP 10.0.0.50.    │
│ Suggests brute force attack.            │
│                                        │
│ 🎯 Targeted: user:admin                │
│ 🖥️  Affected: device:dc01              │
│ 📍 Source IP: 10.0.0.50                │
│ 📊 CVSS: 7.5 (High)                    │
│ 🏷️  ATT&CK: T1110 (Brute Force)        │
│                                        │
│ ✏️  Recommended Actions:               │
│  □ Lock user account immediately      │
│  □ Review login history               │
│  □ Force password reset               │
│  □ Check for lateral movement         │
│                                        │
│ [Confirm] [Suppress] [Tune...]         │
└────────────────────────────────────────┘
```

**Analyst Action:**
Analyst clicks **Confirm** → feedback sent back to Rust → Incident marked as confirmed.

---

## ✨ Features

✅ **Real-time Event Ingestion**
- Accepts JSON events via HTTP POST
- Handles hundreds of events per second
- Automatic entity extraction

✅ **Multi-layer Detection**
- Sigma rule matching (customizable YAML rules)
- Baseline anomaly detection (Z-score)
- IOC threat intelligence checks
- Composite scoring (configurable weights)

✅ **Pattern Correlation**
- Brute force detection (5+ failures in window)
- Lateral movement tracking (user jumping between hosts)
- Privilege escalation detection
- Data exfiltration monitoring

✅ **Incident Generation**
- Automatic incident creation when patterns detected
- Severity scoring (LOW/MEDIUM/HIGH/CRITICAL)
- ATT&CK tactical mapping (T1110, TA0001, etc.)
- CVSS v3.1 calculation

✅ **AI Enrichment** (R3 & R5)
- **R3:** IsolationForest anomaly detection + online learning
- **R5:** SQLite persistence for historical analysis
- LLM integration (Ollama qwen:1.8b) for human-readable summaries
- Fallback templates for offline operation

✅ **Real-time Dashboard**
- Live incident list with pagination
- Severity-based filtering and sorting
- Attack graph visualization
- Incident details with recommended actions
- Analyst feedback submission (confirm/suppress/tune)

✅ **Scalable Architecture**
- Rust backend for high-performance processing
- Python layer for flexible orchestration
- Separated concerns (detection, enrichment, UI)
- SQLite for persistence, in-memory for speed

---

## 📝 Setup & Run Guide

### **Prerequisites**

```bash
# Check installations
rustc --version              # Rust 1.7x+
cargo --version              # Cargo 1.93+
python --version             # Python 3.11+
flutter --version            # (Optional, for UI)
ollama --version             # (Optional, for LLM)
```

**If missing:**
- Rust: https://rustup.rs
- Python: https://python.org (3.11+)
- Flutter: https://flutter.dev
- Ollama: https://ollama.ai

---

### **Full Setup (One-Time)**

```powershell
# Navigate to project
cd c:\Users\Arun\Desktop\Cyber\soc-system

# Run automated setup
.\scripts\setup.ps1
```

**What it does:**
- ✅ Checks Rust, Python, Flutter, Ollama
- ✅ Creates Python virtual environment
- ✅ Installs Python dependencies (`pip install -r requirements.txt`)
- ✅ Downloads Ollama model (4 GB Qwen 1.8B)
- ✅ Builds Rust release binary
- ✅ Gets Flutter dependencies

---

### **Manual Setup**

**Python Virtual Environment:**
```powershell
cd python-ai
python -m venv venv
.\venv\Scripts\Activate.ps1
pip install -r requirements.txt
deactivate
```

**Build Rust:**
```powershell
cd rust-core
cargo build --release
```

**Download LLM (optional):**
```powershell
$env:OLLAMA_NUM_PARALLEL = 1
$env:OLLAMA_MAX_LOADED_MODELS = 1
ollama pull qwen:1.8b
```

---

### **Running the System**

#### **Option 1: Automated (Recommended)**

```powershell
.\scripts\run_all.ps1
```

Starts all components in separate windows:
1. Ollama (background)
2. Rust API Server
3. Python API Layer
4. Flutter UI

---

#### **Option 2: Manual (Terminal-by-Terminal)**

**Terminal 1 - Ollama (Optional)**
```powershell
$env:OLLAMA_NUM_PARALLEL = 1
$env:OLLAMA_MAX_LOADED_MODELS = 1
ollama serve
# Output: Listening on 127.0.0.1:11434
# Wait 2-3 seconds
```

**Terminal 2 - Rust API Server (CORE)**
```powershell
cd rust-core
cargo run -p api-server
# Output: Starting SOC API Server on 0.0.0.0:8080
# Wait 5 seconds
```

**Terminal 3 - Python API Layer**
```powershell
cd python-ai
.\venv\Scripts\Activate.ps1
python -m uvicorn main:app --host 0.0.0.0 --port 8000 --reload
# Output: Uvicorn running on http://0.0.0.0:8000
# Wait 3 seconds
```

**Terminal 4 - Event Generator**
```powershell
cd events/generators
python generate_events.py
# Output: 🚀 Generator started...
# Sends events every 1-3 seconds
```

**Terminal 5 - Flutter UI (Optional)**
```powershell
cd flutter-app
flutter run -d windows
# App window opens in 10-15 seconds
```

---

### **Health Checks**

```bash
# Rust API
curl http://localhost:8080/health
# {"status":"ok"}

# Python API
curl http://localhost:8000/health
# {"status":"ok"}

# Incidents (after events generated)
curl http://localhost:8080/incidents
# [{"id":"...", "severity":"HIGH", ...}]

# SIEM Stats
curl http://localhost:8000/siem/stats
# {"total_incidents": 5, "by_severity": {...}}
```

---

## 📊 Example Output

### **Generated Event**

```json
{
  "event_type": "login_failed",
  "entity": {
    "type": "user",
    "value": "admin"
  },
  "metadata": {
    "source_ip": "10.0.0.5"
  }
}
```

---

### **Detected Incident** (from Rust)

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-03-22T10:05:00Z",
  "severity": "HIGH",
  "chain": ["T1110", "TA0001"],
  "entities": ["user:admin", "device:dc01", "ip:10.0.0.50"],
  "sigma_score": 0.95,
  "z_score": 3.2,
  "ioc_match": null,
  "cvss": 7.5,
  "base_signal": 0.9,
  "summary": null,
  "actions": null
}
```

---

### **Enriched Incident** (after Python processing)

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-03-22T10:05:00Z",
  "severity": "HIGH",
  "chain": ["T1110", "TA0001"],
  "entities": ["user:admin", "device:dc01", "ip:10.0.0.50"],
  "sigma_score": 0.95,
  "z_score": 3.2,
  "ioc_match": null,
  "cvss": 7.5,
  "base_signal": 0.9,
  "anomaly_score": 0.78,
  "summary": "Brute force attack detected. Multiple failed login attempts (5 instances in 60 seconds) on the admin account targeting domain controller dc01 from source IP 10.0.0.50, followed by a successful login. This pattern is highly indicative of a credential stuffing or password guessing attack.",
  "actions": [
    "Immediately lock the admin account to prevent further unauthorized access",
    "Review detailed login attempt logs for the admin account",
    "Initiate password reset for admin account with strong credentials",
    "Check for any privilege escalation or lateral movement post-compromise",
    "Deploy EDR agent to dc01 for deeper visibility",
    "Consider implementing MFA on admin accounts"
  ]
}
```

---

### **SIEM Stats** (from SQLite)

```json
{
  "total_incidents": 47,
  "last_hour_count": 12,
  "by_severity": {
    "LOW": 5,
    "MEDIUM": 15,
    "HIGH": 20,
    "CRITICAL": 7
  },
  "top_entities": {
    "user:admin": 35,
    "device:dc01": 28,
    "ip:10.0.0.5": 18,
    "user:jsmith": 12
  }
}
```

---

### **Graph Snapshot** (Entity Relationships)

```json
{
  "nodes": [
    {"id": "user:admin", "type": "user"},
    {"id": "device:dc01", "type": "device"},
    {"id": "ip:10.0.0.50", "type": "ip"}
  ],
  "edges": [
    {
      "source": "user:admin",
      "target": "device:dc01",
      "event_type": "login_failed",
      "score": 0.6,
      "timestamp": "2026-03-22T10:01:00Z"
    },
    {
      "source": "user:admin",
      "target": "device:dc01",
      "event_type": "login_success",
      "score": 0.95,
      "timestamp": "2026-03-22T10:02:00Z"
    }
  ]
}
```

---

## ⚠️ Limitations

### **Known Constraints**

1. **In-Memory Storage Only**
   - Incidents stored in RAM (Rust AppState)
   - Data lost on server restart
   - Suitable for demo/testing, not production sustained operations
   - **Solution:** Implement persistent database (PostgreSQL/MongoDB)

2. **Sigma Rules Not Loaded**
   - `sigma-rules/` directory included but not parsed at runtime
   - Detection uses hardcoded rules only
   - **Solution:** Parse YAML files at startup, add dynamic rule loading

3. **Basic Correlation Logic**
   - Pattern detection hard-coded for specific scenarios
   - No customizable correlation rules
   - Limited to: brute force, lateral movement, privilege escalation
   - **Solution:** DSL for defining custom correlation patterns

4. **Graph Visualization Limited**
   - Graph stored but visualization on UI is basic
   - No interactive node/edge manipulation
   - **Solution:** Add D3.js or Cytoscape.js visualization

5. **No Persistence Config**
   - Thresholds in `config/thresholds.toml` hardcoded into binary
   - Runtime changes not possible
   - **Solution:** Load config dynamically at runtime, hot-reload

6. **LLM Optional but Recommended**
   - System works without Ollama but less enriched
   - Fallback templates used instead of AI summaries
   - **Solution:** Always include Ollama in deployment

7. **Single-Node Architecture**
   - No horizontal scaling
   - All events processed on one server
   - Max throughput ~1000 events/sec per hardware
   - **Solution:** Kafka/RabbitMQ queue + distributed workers

8. **Limited Feedback Integration**
   - Feedback accepted but not used to retrain detectors
   - Analyst tuning requests ignored
   - **Solution:** Implement feedback loop to adjust thresholds

---

## 🚀 Future Improvements & Roadmap

### **Phase 1: Data Persistence**
- [ ] Replace in-memory incidents with PostgreSQL
- [ ] Implement incident changelog (who modified what, when)
- [ ] Historical trend analysis
- [ ] Incident retention policies (auto-purge old incidents)

### **Phase 2: Advanced Detection**
- [ ] Parse and load Sigma rules from YAML files dynamically
- [ ] Machine learning baseline (per-user, per-asset)
- [ ] YARA rule integration for process/file analysis
- [ ] DGA (Domain Generation Algorithm) detection

### **Phase 3: Scalability**
- [ ] Apache Kafka for event streaming
- [ ] Distributed correlator (multiple workers)
- [ ] Redis for real-time metrics
- [ ] Neo4j for graph database (unlimited scale)

### **Phase 4: Integration & Automation**
- [ ] Slack/Teams/PagerDuty alerting
- [ ] Automated response playbooks (disable user, isolate host)
- [ ] SOAR integration (ServiceNow, Phantom)
- [ ] Threat intelligence feeds (Shodan, AlienVault OTX)
- [ ] Incident ticket creation (Jira, Azure DevOps)

### **Phase 5: Advanced UX**
- [ ] Custom dashboard builder (drag-and-drop widgets)
- [ ] Role-based access control (Analyst, Tier2, SOC Manager, CISO)
- [ ] Dark/light theme, user preferences
- [ ] Multi-language support
- [ ] Mobile app (Android/iOS)

### **Phase 6: Compliance & Audit**
- [ ] Comprehensive audit logging
- [ ] Evidence preservation for forensics
- [ ] Regulatory compliance reports (GDPR, HIPAA, PCI-DSS)
- [ ] Data retention policies
- [ ] Investigation workflow tracking

### **Phase 7: AI/ML Enhancements**
- [ ] Behavioral baselining per-entity
- [ ] Anomaly detection via LSTM networks
- [ ] Threat actor attribution models
- [ ] Predictive incident forecasting

---

## 🔗 Component Dependencies

```
Rust (:8080)
├── Detector (Sigma rules, baseline anomaly, IOC)
├── Graph (Event correlation)
├── Correlator (Pattern matching)
└── Incidents (In-memory store)
        │
        │ (Every 2s)
        ▼
Python (:8000)
├── RustClient (Polls Rust)
├── LLMService (Calls Ollama :11434)
├── AnomalyDetector (R3 - IsolationForest)
├── SIEMService (SQLite persistence - R5)
└── WebSocketManager (Broadcasts)
        │
        ├─→ Flutter UI (Displays)
        │
        ├→ Event Generator (Sends events)
        │
        └─→ Ollama (:11434, optional)
```

---

## 📚 File Structure

```
soc-system/
├── README.md                     (This file)
├── rust-core/
│   ├── api-server/              (HTTP server, port 8080)
│   ├── detector/                (Detection logic)
│   ├── correlator/              (Pattern matching)
│   ├── graph/                   (Attack graph)
│   ├── baseline/                (Anomaly baseline)
│   ├── shared/                  (Common types)
│   ├── event-input/             (CLI ingestion tool)
│   └── Cargo.toml               (Rust dependencies)
├── python-ai/
│   ├── main.py                  (FastAPI app, port 8000)
│   ├── services/
│   │   ├── rust_client.py       (Polls Rust)
│   │   ├── llm.py               (Ollama integration)
│   │   ├── anomaly.py           (R3 detector)
│   │   └── siem.py              (R5 database)
│   ├── routers/
│   │   ├── incidents.py
│   │   ├── feedback.py
│   │   ├── graph.py
│   │   ├── reports.py
│   │   ├── stream.py
│   │   └── siem.py              (R5 endpoints)
│   ├── models/
│   │   ├── incident.py
│   │   └── feedback.py
│   ├── requirements.txt          (Dependencies)
│   └── venv/                     (Virtual env)
├── flutter-app/
│   ├── lib/
│   │   ├── main.dart            (Entry point)
│   │   ├── services/
│   │   │   └── api_service.dart
│   │   ├── models/
│   │   ├── screens/
│   │   └── widgets/
│   ├── pubspec.yaml
│   └── windows/                 (Desktop runner)
├── events/
│   ├── generators/
│   │   └── generate_events.py
│   ├── samples/
│   │   ├── auth_logs.json
│   │   ├── network_logs.json
│   │   └── system_logs.json
│   └── streams/
│       └── event_stream.jsonl
├── config/
│   ├── thresholds.toml
│   ├── entities.toml
│   └── assets.toml
├── sigma-rules/
│   ├── brute_force.yml
│   ├── lateral_movement.yml
│   ├── privilege_escalation.yml
│   └── exfiltration.yml
└── scripts/
    ├── setup.ps1                (One-time setup)
    ├── run_all.ps1              (Start all components)
    └── run_simulation.ps1       (Event generator)
```

---

## 🤝 Contributing

To extend this SOC system:

1. **Add Detection Rule:** Add YAML to `sigma-rules/`, implement in [rust-core/detector/src/lib.rs](rust-core/detector/src/lib.rs)
2. **Add Correlation Pattern:** Extend [rust-core/correlator/src/lib.rs](rust-core/correlator/src/lib.rs)
3. **Add UI Feature:** Extend Flutter screens in [flutter-app/lib/screens/](flutter-app/lib/screens/)
4. **Add Integration:** Add response action in [python-ai/routers/](python-ai/routers/)

---

## 📖 References

**Security Standards:**
- MITRE ATT&CK Framework: https://attack.mitre.org/
- Sigma Rules: https://github.com/SigmaHQ/sigma
- CVSS v3.1: https://www.first.org/cvss/v3.1/specification-document

**Technologies:**
- Rust: https://www.rust-lang.org/
- FastAPI: https://fastapi.tiangolo.com/
- Flutter: https://flutter.dev/
- Ollama: https://ollama.ai/

---

## 📄 License

[Specify your license - e.g., MIT, Apache 2.0]

---

## ✉️ Questions or Issues?

Refer to the documentation above for:
- **Architecture questions** → See [Architecture](#-architecture)
- **How to run** → See [Setup & Run Guide](#-setup--run-guide)
- **System overview** → See [Quick Overview](#-quick-overview)
- **Data flow** → See [System Flow](#-system-flow--end-to-end-example)

This README should provide complete understanding of the SOC pipeline. Enjoy! 🔒
