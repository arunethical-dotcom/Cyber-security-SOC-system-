# SOC System - Security Operations Center Pipeline

A **multi-layered security detection and orchestration system** that ingests security events, detects attack patterns, correlates incidents, and displays them in a real-time dashboard.

This project demonstrates a complete **SOC (Security Operations Center) pipeline** using Rust for high-performance detection and Python for AI-driven enrichment.

---

## 📋 Quick Overview

| Component           | Purpose                                        | Port              | Language         |
| ------------------- | ---------------------------------------------- | ----------------- | ---------------- |
| **Rust API Server** | Event ingestion, detection, correlation engine | 8080              | Rust             |
| **Python AI Layer** | API bridge, enrichment, LLM integration        | 8000              | Python           |
| **Flutter UI**      | Real-time incident dashboard                   | 8000 (via Python) | Dart/Flutter     |
| **Ollama LLM**      | AI-powered incident summarization              | 11434             | External Service |
| **Event Generator** | Simulates attack scenarios and normal events   | -                 | Python           |

---

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     INCIDENT DETECTION PIPELINE                 │
└─────────────────────────────────────────────────────────────────┘

[Event Sources]
      │
      ├─→ Auth Logs (login attempts, failures)
      ├─→ Network Logs (connections, port scans)
      ├─→ System Logs (process execution, privilege escalation)
      └─→ Generated Simulation (attack scenarios)
            │
            ▼
    ┌──────────────────────────────────────────┐
    │  RUST CORE - Event Ingest & Detection   │ :8080
    ├──────────────────────────────────────────┤
    │  [Event Input]                          │
    │    • JSON/JSONL parsing                 │
    │    • Entity resolution (user, IP, host) │
    │                                         │
    │  [Detector]                             │
    │    • Sigma rule matching                │
    │    • Baseline anomaly detection         │
    │    • IOC (Indicator of Compromise)      │
    │    • Multi-scoring algorithm            │
    │                                         │
    │  [Event Graph]                          │
    │    • Entity relationships                │
    │    • Attack chain visualization          │
    │    • TTL-based edge pruning            │
    │                                         │
    │  [Correlator]                           │
    │    • Pattern detection                  │
    │    • Attack classification              │
    │    • Incident generation                │
    │                                         │
    │  [API Endpoints]                        │
    │    • GET /incidents                    │
    │    • GET /graph/snapshot                │
    │    • POST /feedback                     │
    └──────────────────────────────────────────┘
            │
            ▼
    ┌──────────────────────────────────────────┐
    │   PYTHON LAYER - Orchestration & AI     │ :8000
    ├──────────────────────────────────────────┤
    │  [Rust Client]                          │
    │    • Polls Rust :8080 for incidents    │
    │    • Manages connection lifecycle       │
    │                                         │
    │  [LLM Service]                          │
    │    • Calls Ollama for summarization     │
    │    • Generates investigation prompts    │
    │    • Fallback templates included       │
    │                                         │
    │  [WebSocket Manager]                    │
    │    • Broadcasts incidents to clients    │
    │    • Handles multiple connections       │
    │                                         │
    │  [API Routers]                          │
    │    • /incidents → fetch + filter       │
    │    • /reports → generate analysis      │
    │    • /feedback → incident tune/suppress │
    │    • /graph → visualization data        │
    │    • /stream → WebSocket bridge         │
    └──────────────────────────────────────────┘
            │
            ▼
    ┌──────────────────────────────────────────┐
    │    FLUTTER UI - Real-time Dashboard     │
    ├──────────────────────────────────────────┤
    │  • Incident list with severity          │
    │  • Event graph visualization            │
    │  • Filtering & sorting                  │
    │  • Real-time WebSocket updates          │
    │  • Feedback submission                  │
    └──────────────────────────────────────────┘
```

---

## 📊 What is a SOC System?

A **Security Operations Center (SOC)** is the nerve center of cybersecurity defense. It:

1. **Ingests** security events from logs, networks, endpoints
2. **Detects** suspicious patterns using rules and machine learning
3. **Correlates** related events into coherent incidents
4. **Alerts** security teams to investigate
5. **Enriches** incidents with context for faster response

### Real-world SOC Pipeline Example

```
User logs in 5 times with wrong password (10 seconds apart)
  ↓
Event Ingestion: 5 "login_failed" events parsed
  ↓
Detection: Sigma rule matches "brute_force_attempt" (5+ failures in 60s)
  ↓
Scoring: Each event gets sigma_score=0.8 based on rule weight
  ↓
Graph: Events inserted as edges: user:admin → device:dc01
  ↓
Correlation: Pattern detector sees 5 events in 60s window from same source
  ↓
Incident Created: "BruteForceSuccess" incident with HIGH severity
  ↓
UI Display: Analyst sees "High severity incident: Brute force attempt on dc01"
  ↓
Analyst Action: Blocks user, resets password
```

---

## 🔄 System Flow - Detailed

### Step-by-Step Event Processing

```
1. EVENT GENERATION
   └─ Event sources create security events
      Example: {"event_type": "login_failed", "username": "admin",
                "source_ip": "10.0.0.50", "destination_host": "dc01"}

2. EVENT INGESTION (Rust: event-input)
   └─ Parse JSON/JSONL format
   └─ Extract fields: username, IP, hostname, timestamp
   └─ Resolve entities: "admin" → user:admin, "10.0.0.50" → ip:10.0.0.50
   └─ Create CanonicalEvent (standardized format)

3. DETECTION (Rust: detector module)
   └─ For each CanonicalEvent:
      ├─ Check Sigma rules (condition, logsource, category)
      ├─ Calculate sigma_score (rule match weight)
      ├─ Compare against baseline for anomaly detection
      ├─ Check IOC database for known-bad IPs/users
      └─ Blend scores: 0.5*sigma + 0.3*anomaly + 0.2*ioc = final_score

4. EVENT GRAPH INSERTION (Rust: graph module)
   └─ Insert entity nodes: user:admin, device:dc01
   └─ Create directed edge: user:admin → device:dc01
   └─ Edge metadata: timestamp, event_type, score, sigma_rule_id
   └─ Result: Visual attack chain of who accessed what

5. CORRELATION (Rust: correlator module)
   └─ Query graph for patterns in 600s window:
      ├─ Brute Force: 5+ login_failed from same source → user
      ├─ Lateral Movement: user jumping between devices
      ├─ Privilege Escalation: process escalate events on admin account
      └─ Exfiltration: large data transfer from high-value asset
   └─ Create Incident with:
      ├─ ID: uuid
      ├─ Severity: HIGH/CRITICAL based on pattern type
      ├─ CVSS: 7.5 (brute force) to 9.0 (exfiltration)
      ├─ Chain: ATT&CK tactics (T1110, TA0001, etc.)
      └─ Entities involved: [user:admin, device:dc01, ip:10.0.0.50]

6. PYTHON ENRICHMENT (Python: LLM service)
   └─ Poll Rust :8080/incidents every 2 seconds
   └─ For each new incident:
      ├─ Send to Ollama LLM with incident JSON
      ├─ LLM generates: summary, incident_type, recommended_actions
      ├─ On LLM fail: Use fallback template
      └─ Store enriched incident

7. PYTHON BROADCAST (WebSocket)
   └─ Broadcast incident to all connected UI clients in real-time
   └─ Clients see incident within 2-3 seconds of detection

8. UI DISPLAY (Flutter)
   └─ Receive incident via WebSocket
   └─ Add to incident list
   └─ Render with:
      ├─ Severity badge (color-coded)
      ├─ Entity relationships
      ├─ ATT&CK tactics
      ├─ CVSS score
      └─ Recommended actions
```

---

## 🔧 Core Concepts Explained

### 1. Event Ingestion

**What**: Converting raw logs into a standardized format the system understands.

**Why**: Different log sources (Windows Events, syslog, etc.) have different formats. We normalize them.

**Example**:

```
Raw Event: "2026-03-21 10:05:00 user=admin src=10.0.0.50 dst=dc01 status=failed"
           ↓ Parsing & Normalization
Canonical: {
  src_entity: Entity(User, "user:admin"),
  dst_entity: Entity(Device, "device:dc01"),
  event_type: Login,
  raw_fields: {status: "failed", source_ip: "10.0.0.50"},
  timestamp: 2026-03-21T10:05:00Z,
  score: 0.0
}
```

**Code Location**: [rust-core/collector/src/lib.rs](rust-core/collector/src/lib.rs), [rust-core/event-input/src/main.rs](rust-core/event-input/src/main.rs)

---

### 2. Detection Logic

**What**: Scoring events based on rules and baselines.

**Why**: Not all events are malicious. We need to distinguish normal from suspicious.

**How It Works**:

```
For event: "user:admin login to device:dc01 at 10:05am"

Sigma Rules Check:
  ├─ Rule: "admin_login" → condition matches → score += 0.3
  ├─ Rule: "after_hours_login" → no match → skip
  └─ Rule: "multiple_failures_before_success" → no match → skip
  └─ final sigma_score = 0.3

Baseline Check:
  ├─ Historical: "admin typically logs in 9am-5pm, Mon-Fri"
  ├─ Current: "10:05am on Friday" → normal time
  └─ anomaly_score = 0.1 (low, within normal patterns)

IOC Check:
  ├─ Known bad IPs: ["1.1.1.1", "2.2.2.2"] (IP:10.0.0.50 not in list)
  └─ ioc_score = 0.0

Final Composite Score:
  0.5 * sigma_score(0.3) +
  0.3 * anomaly_score(0.1) +
  0.2 * ioc_score(0.0) = 0.18/1.0

Decision: Score 0.18 < alert_threshold(0.7) → NO ALERT (normal event)
```

**Code Location**: [rust-core/detector/src/lib.rs](rust-core/detector/src/lib.rs)

**Thresholds Config**: [config/thresholds.toml](config/thresholds.toml)

---

### 3. Event Graph

**What**: A directed graph where nodes are entities (users, devices, IPs) and edges are events.

**Why**: Visualizes attack chains and enables pattern correlation.

**Example Attack Chain**:

```
Graph visualization:

  user:attacker → (scan) → device:server01
                          (lateral_move) → device:server02
                                          (escalate) → device:dc01

Each edge contains:
  • timestamp: when event occurred
  • event_type: scan, lateral_move, escalate
  • score: detection confidence (0.0-1.0)
  • sigma_rule_id: which rule triggered

This shows: Attacker scanned server, moved laterally, escalated on DC
```

**Code Location**: [rust-core/graph/src/lib.rs](rust-core/graph/src/lib.rs)

---

### 4. Correlation & Pattern Detection

**What**: Finding multi-event patterns that indicate coordinated attacks.

**Why**: A single login failure is normal. 5 login failures in 60 seconds is suspicious.

**Example Patterns**:

```
BRUTE FORCE:
  Pattern: 5+ login_failed events from same source to same user in 60s
  Incident: BruteForceSuccess, severity HIGH, CVSS 7.5
  Example: 10.0.0.50 tries admin password 5 times rapidly → flagged

LATERAL MOVEMENT:
  Pattern: user jumps between 2+ devices via login/process in 15 minutes
  Incident: LateralMove, severity HIGH, CVSS 8.0
  Example: user:admin logs into server01, then server02 → flagged

EXFILTRATION:
  Pattern: High-value user/device sends >1GB data in 10 minutes (z-score > 4.0)
  Incident: ExfilCandidate, severity CRITICAL, CVSS 9.0
  Example: user:admin sends 5GB data → flagged

PRIVILEGE ESCALATION:
  Pattern: Non-admin user triggers escalate event on admin account
  Incident: PrivilegeEscalate, severity MEDIUM, CVSS 6.5
  Example: user:john escalates to admin:root → flagged
```

**Code Location**: [rust-core/correlator/src/lib.rs](rust-core/correlator/src/lib.rs)

**Thresholds**: [config/thresholds.toml](config/thresholds.toml) - controls pattern window sizes and thresholds

---

### 5. Incidents

**What**: A generated alert representing a detected security concern.

**Why**: Raw events are too noisy; incidents are actionable security findings.

**Incident Structure**:

```rust
Incident {
  id: "550e8400-e29b-41d4-a716-446655440000",      // UUID
  timestamp: 2026-03-21T10:05:00Z,                  // When detected
  severity: "HIGH",                                  // HIGH/CRITICAL/MEDIUM/LOW
  chain: ["T1110", "TA0001"],                        // ATT&CK tactics/techniques
  entities: ["user:admin", "device:dc01", "ip:10.0.0.50"],  // Involved actors
  sigma_score: 0.8,                                  // Sigma rule match quality
  z_score: 2.5,                                      // Anomaly deviation (std devs)
  ioc_match: Some("ip:10.0.0.50"),                   // Matched known-bad
  cvss: 7.5,                                         // Common Vulnerability Scoring
  base_signal: 0.8,                                  // Original detection score
  summary: "5 failed logins to admin account...",    // Human-readable (LLM-generated)
  actions: ["Block user", "Reset password", ...]    // Recommended responses
}
```

**Code Location**: [python-ai/models/incident.py](python-ai/models/incident.py), [rust-core/correlator/src/lib.rs](rust-core/correlator/src/lib.rs)

---

## 🦀 Rust Backend - Deep Dive

### Purpose

High-performance, memory-safe event processing:

- Ingest 1000s of events per second
- Real-time detection without data loss
- Minimal CPU/memory overhead

### Architecture

```
┌─────────────────────────────────────────────┐
│         Rust Backend (api-server)           │ :8080
├─────────────────────────────────────────────┤
│                                             │
│  Shared Module                              │
│  ├─ CanonicalEvent (event datastructure)   │
│  ├─ Entity, EntityKind (user/IP/device)    │
│  ├─ EventType (Login/Spawn/Connect/etc)    │
│  └─ Event Bus (tokio broadcast channel)    │
│                                             │
│  Event Input (collector/event-input)       │
│  ├─ Parse JSON/JSONL from file             │
│  ├─ EntityResolver (alias mapping)         │
│  ├─ Normaliser (extract fields)            │
│  └─ Output: CanonicalEvent stream          │
│                                             │
│  Detector Module                           │
│  ├─ Load Sigma rules from .yml files       │
│  ├─ SigmaRule.parse_yaml()                 │
│  ├─ SigmaRule.evaluate(event)              │
│  ├─ Calculate sigma_score                  │
│  ├─ Baseline store (historical patterns)   │
│  ├─ Anomaly scoring (z-score calculation)  │
│  ├─ IOC bloom filter (known-bad IPs)       │
│  └─ Output: DetectionResult {score}        │
│                                             │
│  Event Graph                               │
│  ├─ DiGraph: nodes = entities              │
│  ├─ Edges = events with metadata           │
│  ├─ TTL-based pruning (1800s)              │
│  ├─ Async pruner task (runs every 60s)     │
│  └─ Snapshot API (serializable view)       │
│                                             │
│  Correlator Module                         │
│  ├─ Query graph for patterns               │
│  ├─ Time-window based correlation          │
│  ├─ Generate Incidents                     │
│  ├─ In-memory incident store               │
│  └─ Feedback handler (suppress/tune)       │
│                                             │
│  API Endpoints (Axum router)               │
│  ├─ GET  /health                           │
│  ├─ GET  /incidents[?page=1&limit=50]     │
│  ├─ GET  /incidents/{id}                   │
│  ├─ GET  /graph/snapshot                   │
│  └─ POST /feedback {suppress/tune}         │
│                                             │
└─────────────────────────────────────────────┘
```

### Key Files

| File                                                                   | Purpose                                        |
| ---------------------------------------------------------------------- | ---------------------------------------------- |
| [rust-core/shared/src/lib.rs](rust-core/shared/src/lib.rs)             | Common event structures, EntityKind, EventType |
| [rust-core/detector/src/lib.rs](rust-core/detector/src/lib.rs)         | Sigma rule matching, anomaly detection         |
| [rust-core/graph/src/lib.rs](rust-core/graph/src/lib.rs)               | Event graph (DiGraph), TTL pruning             |
| [rust-core/correlator/src/lib.rs](rust-core/correlator/src/lib.rs)     | Pattern matching, incident generation          |
| [rust-core/baseline/src/lib.rs](rust-core/baseline/src/lib.rs)         | Historical baseline store (SQLite)             |
| [rust-core/api-server/src/main.rs](rust-core/api-server/src/main.rs)   | Axum HTTP server, AppState, endpoints          |
| [rust-core/event-input/src/main.rs](rust-core/event-input/src/main.rs) | CLI for ingesting event files                  |
| [rust-core/collector/src/lib.rs](rust-core/collector/src/lib.rs)       | LogCollector, Normaliser, entity resolution    |

### Event Processing Pipeline

```
Raw Log File (JSON/JSONL)
    ↓
event-input (clap CLI, file parsing)
    ↓
collector::Normaliser (extract fields)
    ↓
CanonicalEvent (standardized)
    ↓
Detector (Sigma rules + baseline + IOC)
    ↓
Score {sigma: 0.8, anomaly: 0.2, ioc: 0.0}
    ↓
EventGraph::insert_event() (create nodes/edges)
    ↓
Correlator::correlate() (pattern match)
    ↓
Incident generation (if pattern found)
    ↓
AppState::incidents.push(incident)
    ↓
API GET /incidents (returns Vec<Incident>)
```

### Data Types

```rust
// Event (standardized across system)
pub struct CanonicalEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub src_entity: Entity,           // user:admin, ip:10.0.0.50
    pub dst_entity: Entity,           // device:dc01
    pub event_type: EventType,        // Login, Connect, Spawn, etc.
    pub raw_fields: HashMap<String, String>,  // metadata
    pub score: f32,                   // 0.0-1.0 detection score
}

// Incident (what gets displayed)
pub struct Incident {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub severity: String,             // HIGH, CRITICAL, etc.
    pub chain: Vec<String>,           // ["T1110", "TA0001"]
    pub entities: Vec<String>,        // involved actors
    pub sigma_score: f32,
    pub z_score: f32,
    pub ioc_match: Option<String>,
    pub cvss: f32,
    pub base_signal: f32,
    pub summary: Option<String>,      // AI-generated
    pub actions: Option<Vec<String>>, // recommended actions
}
```

---

## 🐍 Python Backend - Orchestration & AI

### Purpose

- Bridge between Rust (detection) and Flutter (UI)
- Add AI-driven incident enrichment
- Handle WebSocket real-time updates

### Architecture

```
┌─────────────────────────────────────────────┐
│      Python Layer (FastAPI)                 │ :8000
├─────────────────────────────────────────────┤
│                                             │
│  Lifespan Manager                           │
│  ├─ Startup: Connect RustClient             │
│  ├─ Startup: Initialize WebSocketManager    │
│  ├─ Startup: Initialize LLMService          │
│  └─ Shutdown: Close connections             │
│                                             │
│  RustClient (services/rust_client.py)       │
│  ├─ HTTP connection to Rust :8080          │
│  ├─ Poll /incidents endpoint (2s interval) │
│  ├─ Track new incidents (by ID)            │
│  ├─ Fire callbacks on incident received     │
│  └─ Handle disconnections gracefully       │
│                                             │
│  LLMService (services/llm.py)              │
│  ├─ Call Ollama :11434/api/generate        │
│  ├─ Send incident JSON + prompt             │
│  ├─ Get LLM response (summary, actions)     │
│  ├─ JSON validation + error handling        │
│  └─ Fallback templates (if LLM fails)      │
│                                             │
│  WebSocketManager (services/websocket.py)  │
│  ├─ Accept WebSocket connections           │
│  ├─ Broadcast incidents to all clients      │
│  ├─ Handle disconnections                   │
│  └─ JSON serialization                      │
│                                             │
│  API Routers (FastAPI routes)               │
│  ├─ incidents.router:                       │
│  │  ├─ GET /incidents [page, limit, severity]│
│  │  └─ GET /incidents/{id}                  │
│  ├─ reports.router:                         │
│  │  ├─ GET /reports/{report_id}            │
│  │  └─ GET /reports/{report_id}/pdf        │
│  ├─ feedback.router:                        │
│  │  └─ POST /feedback (suppress/tune)      │
│  ├─ graph.router:                           │
│  │  └─ GET /graph/snapshot                  │
│  └─ stream.router:                          │
│     └─ WebSocket /stream                    │
│                                             │
│  Global State                               │
│  ├─ rust_client: Connects to Rust backend  │
│  ├─ intel_service: Data enrichment         │
│  └─ ws_manager: Manages WebSocket conns    │
│                                             │
└─────────────────────────────────────────────┘
```

### Key Files

| File                                                                   | Purpose                                    |
| ---------------------------------------------------------------------- | ------------------------------------------ |
| [python-ai/main.py](python-ai/main.py)                                 | FastAPI app, lifespan, router registration |
| [python-ai/services/rust_client.py](python-ai/services/rust_client.py) | HTTP client polling Rust backend           |
| [python-ai/services/llm.py](python-ai/services/llm.py)                 | Ollama integration for incident enrichment |
| [python-ai/services/websocket.py](python-ai/services/websocket.py)     | WebSocket broadcast manager                |
| [python-ai/routers/incidents.py](python-ai/routers/incidents.py)       | GET /incidents, /incidents/{id}            |
| [python-ai/routers/reports.py](python-ai/routers/reports.py)           | Report generation (PDF)                    |
| [python-ai/routers/feedback.py](python-ai/routers/feedback.py)         | Feedback submission                        |
| [python-ai/routers/graph.py](python-ai/routers/graph.py)               | Graph visualization data                   |
| [python-ai/routers/stream.py](python-ai/routers/stream.py)             | WebSocket endpoints                        |
| [python-ai/models/incident.py](python-ai/models/incident.py)           | Pydantic models                            |

### Data Flow

```
Startup:
  FastAPI starts
    ├─ RustClient connects to Rust :8080
    ├─ WebSocketManager initialized
    └─ LLMService initialized

Runtime:
  RustClient polls :8080/incidents every 2 seconds
    ├─ Receives new Incident from Rust
    ├─ Calls LLMService.generate_summary(incident)
    │   ├─ Sends to Ollama :11434/api/generate
    │   ├─ Gets JSON response {summary, actions, ...}
    │   └─ Returns LLMOutput
    │
    └─ Broadcasts to all WebSocket clients
        └─ websocket.broadcast({incident + llm_output})

Client Request: GET /incidents
    └─ Returns [Incident] from Rust (cached or fresh polling)

Client Request: WebSocket /stream
    └─ Gets real-time incident broadcasts
```

### LLM Enrichment Example

```
Rust sends Incident:
{
  "severity": "HIGH",
  "chain": ["T1110", "TA0001"],
  "entities": ["user:admin", "device:dc01"],
  "sigma_score": 0.8,
  "cvss": 7.5,
  "base_signal": 0.8
}

LLM Prompt:
  "You are a security analyst. Analyze this incident and respond
   with JSON: {incident_type, severity, summary, recommended_actions,
   confidence_caveat}"

LLM Response:
{
  "incident_type": "brute_force_attack",
  "severity": "HIGH",
  "summary": "Multiple failed login attempts detected on admin account
             to domain controller. Pattern suggests brute force attack.",
  "recommended_actions": [
    "Immediately lock admin account",
    "Review failed login logs for source IPs",
    "Check for successful logins after failures",
    "Deploy EDR agent to dc01 if not already present"
  ],
  "confidence_caveat": "Automated analysis - manual verification recommended"
}

Final Incident (displayed in UI):
  Incident {
    severity: "HIGH",
    summary: "Multiple failed login attempts...",    // from LLM
    actions: ["Immediately lock admin account", ...] // from LLM
  }
```

---

## 🎨 Flutter Frontend - Real-Time Dashboard

### Purpose

Provide security analysts with:

- Real-time incident list
- Filtering by severity
- Event graph visualization
- Incident details and recommended actions
- Feedback submission (confirm, suppress, tune)

### Architecture

```
┌──────────────────────────────────────┐
│     Flutter Desktop (Windows)         │
├──────────────────────────────────────┤
│                                      │
│  ApiService (api_service.dart)       │
│  ├─ baseUrl: http://localhost:8000  │
│  ├─ Dio HTTP client (10s connect)   │
│  ├─ getIncidents(page, limit, sev)  │
│  ├─ getIncidentById(id)             │
│  ├─ getGraphSnapshot()              │
│  ├─ submitFeedback(feedback)        │
│  └─ getReportPdf(id)                │
│                                      │
│  Screens (lib/screens/)              │
│  ├─ Home Screen:                    │
│  │  ├─ Incident list (paginated)    │
│  │  ├─ Severity filter dropdown     │
│  │  └─ Real-time refresh            │
│  │                                  │
│  ├─ Incident Detail:                │
│  │  ├─ Full incident data           │
│  │  ├─ Involved entities            │
│  │  ├─ ATT&CK chain                │
│  │  ├─ LLM summary (if available)   │
│  │  ├─ Recommended actions          │
│  │  └─ Feedback buttons             │
│  │                                  │
│  ├─ Graph View:                     │
│  │  ├─ Incident entity relationships│
│  │  ├─ Attack chain visualization   │
│  │  └─ Interactive nodes/edges      │
│  │                                  │
│  └─ Feedback Dialog:                │
│     ├─ Confirm incident             │
│     ├─ Suppress incident            │
│     └─ Tune thresholds              │
│                                      │
└──────────────────────────────────────┘
```

### Key Files

| File                                                                                   | Purpose                                  |
| -------------------------------------------------------------------------------------- | ---------------------------------------- |
| [flutter-app/lib/services/api_service.dart](flutter-app/lib/services/api_service.dart) | HTTP client, API calls to Python backend |
| [flutter-app/lib/models/incident.dart](flutter-app/lib/models/incident.dart)           | Incident data model                      |
| [flutter-app/lib/models/feedback.dart](flutter-app/lib/models/feedback.dart)           | Feedback structure                       |
| [flutter-app/lib/screens/](flutter-app/lib/screens/)                                   | UI screens (incidents, graph, feedback)  |
| [flutter-app/lib/main.dart](flutter-app/lib/main.dart)                                 | Entry point, app configuration           |

### API Calls

```dart
// Get incidents with filters
List<Incident> incidents = await apiService.getIncidents(
  page: 1,
  limit: 50,
  severity: "HIGH"  // optional filter
);
// Makes GET /incidents?page=1&limit=50&severity=HIGH

// Get full incident details
Incident detail = await apiService.getIncidentById(incidentId);
// Makes GET /incidents/{incidentId}

// Get event graph visualization
Map<String, dynamic> graph = await apiService.getGraphSnapshot();
// Makes GET /graph/snapshot
// Returns {nodes: [...], edges: [...]}

// Submit feedback
await apiService.submitFeedback(UserFeedback(
  incident_id: id,
  action: "suppress",  // or "confirm", "tune"
  new_threshold: 0.8
));
// Makes POST /feedback with JSON body
```

---

## 🎯 Event Generator - Simulation

### Purpose

Simulate realistic attack scenarios and normal events for testing the system.

### Files

| File                                                                         | Purpose                                  |
| ---------------------------------------------------------------------------- | ---------------------------------------- |
| [events/generators/generate_events.py](events/generators/generate_events.py) | Main generator, creates events           |
| [events/samples/auth_logs.json](events/samples/auth_logs.json)               | Sample auth events (login attempts)      |
| [events/samples/network_logs.json](events/samples/network_logs.json)         | Sample network events (connections)      |
| [events/samples/system_logs.json](events/samples/system_logs.json)           | Sample system events (process execution) |
| [events/streams/event_stream.jsonl](events/streams/event_stream.jsonl)       | Stream of generated events               |

### Example Events

```json
Auth Log - Normal Login:
{
  "timestamp": "2026-03-21T10:00:00Z",
  "event_type": "login_success",
  "username": "jsmith",
  "source_ip": "192.168.1.100",
  "destination_host": "server01",
  "status": "success"
}

Auth Log - Failed Login (single):
{
  "timestamp": "2026-03-21T10:01:00Z",
  "event_type": "login_failed",
  "username": "admin",
  "source_ip": "10.0.0.50",
  "destination_host": "dc01",
  "status": "failed"
}

Brute Force Attack Simulation:
{
  "timestamp": "2026-03-21T10:01:15Z",
  "event_type": "login_failed",
  "username": "admin",
  "source_ip": "10.0.0.50",    // Same source
  "destination_host": "dc01",   // Same target
  "status": "failed"
}
// Repeated 5 times in 60 seconds → triggers Brute Force Incident

Network Log - Port Scan:
{
  "timestamp": "2026-03-21T10:05:00Z",
  "event_type": "connect",
  "source_ip": "10.0.0.50",
  "destination_ip": "192.168.1.0",
  "destination_port": 22,      // Multiple ports
  "direction": "inbound"
}

System Log - Process Execution:
{
  "timestamp": "2026-03-21T10:10:00Z",
  "event_type": "spawn",
  "username": "admin",
  "process_name": "powershell.exe",
  "command_line": "powershell -Command Get-ScheduledTask",
  "parent_process": "explorer.exe"
}

Exfiltration Simulation:
{
  "timestamp": "2026-03-21T10:15:00Z",
  "event_type": "connect",
  "username": "admin",
  "source_ip": "192.168.1.50",
  "destination_ip": "1.1.1.1",      // External
  "bytes_sent": 5368709120,         // 5GB!
  "direction": "outbound"
}
```

### Generator Output

```bash
$ python generators/generate_events.py

🚀 Generator started...
⚠️ Simulating brute force attack...
✅ Sent: {event_type: login_failed, ...} | Status: 200
✅ Sent: {event_type: login_failed, ...} | Status: 200
✅ Sent: {event_type: login_failed, ...} | Status: 200
✅ Sent: {event_type: login_failed, ...} | Status: 200
✅ Sent: {event_type: login_failed, ...} | Status: 200

⚠️ Simulating lateral movement...
✅ Sent: {event_type: connect, ...} | Status: 200
✅ Sent: {event_type: spawn, ...} | Status: 200
```

---

## 🚀 Setup & Running the System

### Prerequisites

```powershell
# Check requirements
cargo --version          # Rust toolchain (installed from rustup.rs)
python --version         # Python 3.11+ (from python.org)
flutter --version        # Flutter SDK (from flutter.dev) - optional for UI
ollama --version         # Ollama LLM service (from ollama.ai)
```

### Initial Setup (First Time)

```powershell
# Navigate to project
cd c:\Users\Arun\Desktop\Cyber\soc-system

# 1. Setup Python environment
python -m venv python-ai\venv
.\python-ai\venv\Scripts\Activate.ps1
pip install -r python-ai\requirements.txt
deactivate

# 2. Build Rust binaries
cd rust-core
cargo build --release
cd ..

# 3. Pull LLM model (first time only)
$env:OLLAMA_NUM_PARALLEL = 1
$env:OLLAMA_MAX_LOADED_MODELS = 1
ollama pull qwen:1.8b    # ~400MB download, one-time

# 4. Verify Flask UI Firebase setup
cd flutter-app
flutter pub get
cd ..
```

### Running the System - Step by Step

**Terminal 1: Start Ollama (LLM Service)**

```powershell
$env:OLLAMA_NUM_PARALLEL = 1
$env:OLLAMA_MAX_LOADED_MODELS = 1
ollama serve

# Output: Listening on 127.0.0.1:11434
```

**Terminal 2: Start Rust API Server (Wait ~2 seconds)**

```powershell
cd rust-core
cargo run -p api-server

# Output: "Starting SOC API Server on 0.0.0.0:8080"
# Wait 5 seconds for server to fully start
```

**Terminal 3: Start Python API Layer (Wait ~1 second)**

```powershell
cd python-ai
python -m uvicorn main:app --host 0.0.0.0 --port 8000 --reload

# Output: "Uvicorn running on http://127.0.0.1:8000"
# Should show: "Rust client started with base URL: http://localhost:8080"
```

**Terminal 4: Start Flutter UI (Optional)**

```powershell
cd flutter-app
flutter run -d windows

# Output: App window opens, connects to :8000
```

**Terminal 5: Generate Events**

```powershell
cd events\generators
python generate_events.py --mode brute_force

# Output: Generator starts sending events
#         Watch Terminal 3 for "Incident detected..."
```

### Quick Health Check

```powershell
# Check all services running
curl http://localhost:8000/health
# Output: {"status":"ok"}

curl http://localhost:8080/health
# Output: {"status":"ok"}

# Check incidents (should be empty initially)
curl http://localhost:8080/incidents
# Output: []

# Wait 10 seconds for events to process...
curl http://localhost:8080/incidents
# Output: [{"id":"...", "severity":"HIGH", ...}]
```

### Full Automated Startup

```powershell
# Run the provided script (starts all components)
.\scripts\run_all.ps1

# This will:
# 1. Start Ollama
# 2. Start Rust API Server
# 3. Start Python API
# 4. Start Flutter UI (if installed)
# 5. You manually run generator in another terminal
```

---

## 📈 Example Scenario - Brute Force Attack Detection

### Scenario

Attacker at IP `10.0.0.50` attempts to brute-force the `admin` account on domain controller `dc01`.

### Event Generation

```
Time    Event Type      Username  Source IP    Target      Status
10:01   login_failed    admin     10.0.0.50    dc01:LOGIN  Failed
10:01   login_failed    admin     10.0.0.50    dc01:LOGIN  Failed
10:01   login_failed    admin     10.0.0.50    dc01:LOGIN  Failed
10:01   login_failed    admin     10.0.0.50    dc01:LOGIN  Failed
10:01   login_failed    admin     10.0.0.50    dc01:LOGIN  Failed
10:02   login_success   admin     10.0.0.50    dc01:LOGIN  Success
```

### Step 1: Event Ingestion (Rust)

Generator sends HTTP POST to `http://localhost:8080/events`:

```json
{
  "event_type": "login_failed",
  "username": "admin",
  "source_ip": "10.0.0.50",
  "destination_host": "dc01"
}
```

Event-input CLI parses 6 events into CanonicalEvents:

```rust
CanonicalEvent {
  src_entity: Entity(User, "user:admin"),
  dst_entity: Entity(Device, "device:dc01"),
  event_type: Login,
  raw_fields: {status: "failed"},
  timestamp: 2026-03-21T10:01:00Z
}
// 5 more with timestamps 10:01:15, 10:01:30, 10:01:45, 10:02:00, 10:02:15
```

### Step 2: Detection (Rust Detector)

For each event, detector runs:

**Event 1 (login_failed):**

```
Sigma check: Matches "brute_force" rule? No (only 1 event so far)
Anomaly check: Expected 0.1 logins/min for admin, got 1. Score: 0.2
IOC check: IP 10.0.0.50 not in known-bad list. Score: 0.0
Final: 0.5*0 + 0.3*0.2 + 0.2*0 = 0.06  → Below threshold (0.7)
Decision: NORMAL - not suspicious yet
```

**Event 2-5 (login_failed x4):**

```
Similar scores. Building up in graph but not all crossing threshold yet.
```

**Event 6 (login_success):**

```
Sigma check: Matches "successful_after_failures" rule. Score: 0.9
Anomaly check: Success after 5 failures in 61 seconds. Score: 0.8
IOC check: IP not in list. Score: 0.0
Final: 0.5*0.9 + 0.3*0.8 + 0.2*0 = 0.69
Decision: HIGH SCORE - approaching threshold
```

### Step 3: Graph Building

Events 1-6 inserted as edges in EventGraph:

```
Nodes created:
  - user:admin
  - device:dc01
  - ip:10.0.0.50

Edges:
  user:admin → device:dc01 (event 1, score 0.06, login_failed)
  user:admin → device:dc01 (event 2, score 0.06, login_failed)
  user:admin → device:dc01 (event 3, score 0.06, login_failed)
  user:admin → device:dc01 (event 4, score 0.06, login_failed)
  user:admin → device:dc01 (event 5, score 0.06, login_failed)
  user:admin → device:dc01 (event 6, score 0.69, login_success)

Visual chain: user:admin ←→ device:dc01 (6 connection attempts)
```

### Step 4: Correlation (Rust Correlator)

Correlator queries graph for patterns in 600s window:

```
Pattern matching - BRUTE FORCE DETECTED:
  Query: Find 5+ login_failed edges from same source to same target in 60s

  Results: 5 login_failed edges (timestamps within 60 seconds)
           All from: user:admin
           All to: device:dc01

  Pattern Score: 5 attempts > threshold of 5 ✓
  Time window: 61s < 600s ✓

  → INCIDENT GENERATED
```

### Step 5: Incident Creation

Correlator creates Incident object:

```rust
Incident {
    id: "550e8400-e29b-41d4-a716-446655440000",
    timestamp: 2026-03-21T10:02:15Z,
    severity: "HIGH",
    chain: ["T1110", "TA0001"],  // Brute Force (T1110), Initial Access (TA0001)
    entities: ["user:admin", "device:dc01", "ip:10.0.0.50"],
    sigma_score: 0.69,
    z_score: 3.2,              // 3.2 std devs above baseline
    ioc_match: None,
    cvss: 7.5,                 // CVSS v3.1 score for brute force
    base_signal: 0.69,
    summary: None,             // Will be filled by LLM
    actions: None,             // Will be filled by LLM
}
```

Incident stored in `AppState::incidents`.

### Step 6: Python Polling (Python Backend)

RustClient polling loop:

```python
# Every 2 seconds:
incidents = await self.get_incidents(limit=10)
current_ids = {inc.id for inc in incidents}
new_ids = current_ids - self._last_incident_ids

# Found new incident!
new_ids = {"550e8400-e29b-41d4-a716-446655440000"}

# Trigger callbacks
for callback in self._callbacks:
    await callback(incident)  # Callback = LLM enrichment
```

### Step 7: LLM Enrichment (Python LLMService)

LLMService receives incident, calls Ollama:

```python
prompt = """You are a security analyst. Analyze:
{
  "severity": "HIGH",
  "chain": ["T1110", "TA0001"],
  "entities": ["user:admin", "device:dc01", "ip:10.0.0.50"],
  "sigma_score": 0.69,
  "z_score": 3.2,
  "cvss": 7.5
}

Respond with JSON: {
  incident_type, severity, summary, recommended_actions, confidence_caveat
}"""

response = await ollama.post("http://localhost:11434/api/generate",
    model="qwen:1.8b",
    prompt=prompt
)

result = parse_json(response)
# Returns LLMOutput:
# {
#   incident_type: "brute_force_attack",
#   severity: "HIGH",
#   summary: "Multiple failed login attempts followed by successful..."
#   recommended_actions: [
#     "Immediately lock admin account",
#     "Review login logs for source IPs",
#     ...
#   ]
# }
```

### Step 8: WebSocket Broadcast

```python
enriched_incident = {
    **incident_dict,
    "summary": "Multiple failed login attempts...",
    "recommended_actions": ["Immediately lock admin account", ...]
}

await ws_manager.broadcast(enriched_incident)
# All connected clients receive incident in real-time
```

### Step 9: UI Display

Flutter UI receives WebSocket message:

```dart
WebSocket connection active to :8000/stream

Received incident:
{
  id: "550e8400-e29b-41d4-a716-446655440000",
  severity: "HIGH",
  summary: "Multiple failed login attempts detected on admin account
           to domain controller dc01 from IP 10.0.0.50. Pattern suggests
           coordinated brute force attack.",
  recommended_actions: [
    "Immediately lock admin account",
    "Review login logs for source IPs 10.0.0.50",
    "Check for successful logins after failures",
    "Deploy EDR to dc01"
  ]
}

// UI renders:
┌─────────────────────────────────────────────────┐
│ HIGH SEVERITY INCIDENT                          │
│ ID: 550e8400-e29b-41d4-a716-446655440000       │
│ Detected: 2026-03-21 10:02:15 UTC              │
│                                                 │
│ Multiple failed login attempts detected on      │
│ admin account to domain controller dc01 from    │
│ IP 10.0.0.50. Pattern suggests coordinated     │
│ brute force attack.                            │
│                                                 │
│ Entities: user:admin, device:dc01, ip:10...   │
│ CVSS Score: 7.5                                │
│ ATT&CK: T1110 (Brute Force), TA0001 (Init...)  │
│                                                 │
│ [Confirm] [Suppress] [Tune...]                 │
│                                                 │
│ Recommended Actions:                            │
│  • Immediately lock admin account              │
│  • Review login logs for source IPs             │
│  • Check for successful logins after failures   │
│  • Deploy EDR to dc01                           │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Step 10: Analyst Action

Analyst clicks "Confirm" → Submits feedback → Python backend records → Rust correlator noted for feedback tuning.

---

## ⚠️ Known Limitations & Issues

### 1. **Missing Event Ingestion Endpoint** (Critical)

Currently, the generator sends events to `http://localhost:8080/events` but this
endpoint does NOT exist in the Rust API server. Events are dropped (404).

**Workaround**: Events must be ingested via the CLI tool:

```powershell
cd rust-core
cargo run --bin event-input -- --input ..\events\samples\auth_logs.json
```

**Status**: Blocks real detection - generator won't trigger incidents.

### 2. **Python Routers Are Stubs**

Routers in `python-ai/routers/` return hardcoded stubs instead of actual data:

- `incidents.py` returns `[]` (empty list)
- `graph.py` returns `{nodes: [], edges: []}`
- `reports.py` returns "not implemented"

**Status**: Flutter UI will show empty data. Workaround: Connect directly to Rust :8080.

### 3. **Sigma Rules Not Loaded**

Detector creates empty `Vec<SigmaRule>`. Rules in `sigma-rules/*.yml` are never loaded.

**Status**: Detection runs with zero rules. Baseline anomaly still works.

### 4. **Configuration Files Not Read**

`thresholds.toml` and other configs exist but are never loaded. System uses hardcoded defaults.

**Status**: Threshold tuning via UI won't persist.

### 5. **In-Memory Storage Only**

Incidents stored in RAM (`AppState::incidents`). System restart = all data lost.

**Status**: No persistence. Suitable for demo/testing only.

### 6. **IOC Bloom Filter Empty**

IOC database (known-bad IPs/hashes) is initialized empty and never populated.

**Status**: IOC detection skipped (score always 0.0).

### 7. **Graph Edges Not Pruned in Real-Time**

TTL pruning task runs every 60s. Old edges linger until pruned.

**Status**: Graph grows unbounded until pruning interval. Not critical for short runs.

### 8. **No Feedback Persistence**

Feedback (suppress, tune) is logged but doesn't update thresholds or feedback store.

**Status**: Feedback features don't work.

### 9. **LLM Optional But Recommended**

If Ollama fails, fallback templates used. System still works but less enriched.

**Status**: Graceful degradation. Non-critical.

### 10. **Correlator Window Times Hardcoded**

Correlation time windows (600s, 900s, etc.) are hardcoded constants, not configurable.

**Status**: Can't tune pattern detection without code recompilation.

---

## 🔮 Future Improvements & Roadmap

### Phase 1: Fix Critical Gaps (Next)

- [ ] **Add `/events` HTTP Endpoint** - Enable real event ingestion from generators
- [ ] **Load Sigma Rules from YAML** - Read rules from `sigma-rules/` directory
- [ ] **Load Configuration** - Parse `thresholds.toml` and entity aliases at startup
- [ ] **Implement Python Routers** - Connect Flask routes to Rust backend data
- [ ] **Persistent Storage** - Replace in-memory storage with SQLite/PostgreSQL

### Phase 2: Advanced Detection

- [ ] **Machine Learning Models** - Replace correlation rules with trained models
- [ ] **Real-time Streaming** - Kafka/RabbitMQ instead of polling
- [ ] **Advanced Baselines** - Weekly/seasonal patterns, per-user profiles
- [ ] **YARA Rule Integration** - File/process signature matching
- [ ] **DGA Detection** - Domain generation algorithm detection for C2

### Phase 3: Scalability & Performance

- [ ] **Horizontal Scaling** - Distributed graph store (Neo4j, ArangoDB)
- [ ] **Event Batching** - Buffer + process 1000s of events/sec
- [ ] **Caching Layer** - Redis for frequently accessed data
- [ ] **Async Processing** - Move heavy enrichment off main pipeline
- [ ] **Distributed Tracing** - OpenTelemetry for observability

### Phase 4: User Experience

- [ ] **Threat Intelligence Integration** - Feed external IOC databases
- [ ] **Playbook Automation** - Automated response (block IP, isolate host, etc.)
- [ ] **Custom Dashboards** - Drag-and-drop widget builder
- [ ] **Alert Routing** - Slack/Teams/PagerDuty integration
- [ ] **SOAR Integration** - Orchestration with ServiceNow, Phantom, etc.

### Phase 5: Compliance & Audit

- [ ] **Audit Logging** - Who accessed what, when
- [ ] **Evidence Preservation** - Collect/store forensic data
- [ ] **Reporting** - PDF/CSV exports for compliance (GDPR, HIPAA, PCI-DSS)
- [ ] **Role-Based Access Control** - Admin vs Analyst vs Read-Only
- [ ] **Data Retention** - Configurable retention policies

---

## 📚 Key Technologies

| Component            | Tech Stack           | Why?                                    |
| -------------------- | -------------------- | --------------------------------------- |
| **Event Ingestion**  | Rust + Tokio         | Fast, memory-safe, async I/O            |
| **Detection Engine** | Rust                 | Real-time scoring without GC pauses     |
| **Graph Database**   | Petgraph (in-memory) | Fast relationship queries               |
| **HTTP Server**      | Axum + Tower         | Minimal overhead, composable middleware |
| **API ORM**          | Pydantic             | Type-safe Python API contracts          |
| **LLM Integration**  | Ollama + Qwen        | Opensource, runs locally                |
| **WebSocket**        | FastAPI WebSockets   | Real-time incident push                 |
| **Frontend**         | Flutter              | Cross-platform desktop UI               |
| **CLI Tool**         | Clap                 | Simple event ingestion CLI              |
| **Configuration**    | TOML                 | Human-readable config files             |

---

## 🔗 Component Interaction Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     EVENT LIFECYCLE                         │
└─────────────────────────────────────────────────────────────┘

[Event Source]
      │
      │ JSON event
      ▼
[Generator / CLI] ──[Generator sends POST /:8080/events: (BLOCKED)]──→ [Rust API Server]
                                                                        │
[Event Source Files] ──[event-input CLI reads]──→ [Parser] ──→ [Detector]
   auth_logs.json                                   │            │
   network_logs.json                                │            ├─→ Sigma scoring
   system_logs.json                                 │            ├─→ Baseline scoring
                                                    │            ├─→ IOC matching
                           ┌───────────────────────┘            └→ (score)
                           │                                       │
                           ▼                                       ▼
                    [EventGraph]                           [Correlator]
                    • Insert edge                          • Query graph
                    • Prune old edges                      • Find patterns
                          │                                • Create Incident
                          │                                       │
                          └───────────────────────┬────────────────┘
                                                  │
                                                  ▼
                                        [Python Backend :8000]
                                        • RustClient poll :8080
                                        • LLM enrichment (Ollama :11434)
                                        • WebSocket broadcast
                                        • Flask routes
                                               │
                                               ▼
                                        [Flutter UI]
                                        • Display incidents
                                        • Filter / Sort
                                        • Submit feedback
```

---

## 📖 Further Reading

- **Sigma Rules**: https://github.com/SigmaHQ/sigma
- **ATT&CK Framework**: https://attack.mitre.org/
- **CVSS Calculator**: https://www.first.org/cvss/calculator/3.1
- **Rust Security**: https://owasp.org/www-community/attacks/Rust
- **Petgraph Docs**: https://docs.rs/petgraph/
- **Axum Web Framework**: https://github.com/tokio-rs/axum

---

## 🤝 Contributing

This is an educational project demonstrating SOC architecture. Contributions welcome:

1. Fix the missing `/events` endpoint
2. Load Sigma rules from YAML
3. Implement Python router logic
4. Add persistent storage
5. Improve LLM prompts

---

## 📜 License

[Specify your license - e.g., MIT, Apache 2.0]

---

## ✉️ Questions?

Refer to the analysis in this README for:

- Architecture overview
- Component responsibilities
- Data flow between layers
- How to extend the system

Enjoy exploring the SOC pipeline! 🔒
