# 🚀 Cyber Security SOC System

A real-time, AI-assisted Security Operations Center (SOC) pipeline designed to **detect, correlate, and respond to cyber threats** using a hybrid approach of rule-based detection, machine learning, and graph-based correlation.

---

# 🎯 Overview

This system processes raw security logs and transforms them into **actionable security incidents** by:

* Detecting known and unknown threats
* Correlating events into multi-stage attack chains
* Enriching incidents with AI-generated insights
* Streaming alerts in real-time

---

# ⚡ Key Capabilities

### 🔴 Real-Time Threat Detection

Detects:

* Brute force attacks
* Lateral movement
* Privilege escalation
* Data exfiltration

---

### 🧠 Hybrid Detection Engine

Combines multiple detection strategies:

* **Sigma Rules** → Known attack patterns
* **ML (Isolation Forest)** → Behavioral anomalies
* **IOC Matching** → Known malicious indicators

---

### 🔗 Graph-Based Correlation

Transforms isolated logs into **connected attack chains** using a directed event graph.

---

### 🤖 AI-Powered Enrichment

Uses LLMs to generate:

* Human-readable incident summaries
* Recommended remediation actions

---

### 📊 Real-Time Dashboard

* Live alert streaming via WebSocket
* Timeline + severity visualization
* Attack chain visibility

---

# 🏗️ System Architecture

## 🔄 Data Flow

```
Logs → Ingestion → Detection → Correlation → Incident → AI Enrichment → Dashboard
```

---

## ⚙️ Communication Model

* **Rust ↔ Python** → REST APIs
* **Python → Dashboard** → WebSocket (real-time streaming)

---

## ⚠️ Failure Handling

* LLM failure → fallback templates
* Rust API failure → retry with exponential backoff
* WebSocket disconnect → automatic reconnection

---

# 🧠 Detection Engine

## 🔢 Scoring Model

Each event is scored using:

```
Final Score = (Sigma × 0.5) + (Anomaly × 0.3) + (IOC × 0.2)
```

* Scores are normalized between 0–1
* Weighted fusion balances precision (rules) and recall (ML)

---

## 🚨 Decision Threshold

* **Score > 0.7 → Incident generated**

---

## ⚠️ Design Note

* Rule-based signals can dominate for known attacks
* ML contributes to anomaly detection but is not solely trusted
* Future improvements include adaptive weighting and calibration

---

# 🔬 Deep Dive: Correlation Engine

## ⏱ Time Windows (Sliding)

* Brute Force → 60 seconds
* Lateral Movement → 10–15 minutes
* Privilege Escalation → 5–10 minutes

These windows allow:

* Fast detection of burst attacks
* Correlation of longer multi-stage attacks

---

## 🧩 Graph Model

* **Nodes** → Users, IPs, Devices, Processes
* **Edges** → Events (login, connect, process spawn)

Graph is maintained in-memory with time-based pruning.

---

## 🔍 Traversal Strategy

1. Start from a suspicious node (user/IP)
2. Traverse connected edges within time window (bounded depth)
3. Aggregate event scores
4. Identify repeating or chained patterns

Traversal is constrained to avoid unbounded graph expansion.

---

## 🔗 Attack Chain Formation

Example chain:

```
Login Failure → Successful Login → Process Execution → Privilege Escalation
```

Output includes:

* Attack type
* Timeline
* Entities involved
* Confidence score

---

# 🔄 End-to-End Example

## 📥 Input Logs

```json
[
  {"event_type": "login_failed", "user": "admin", "ip": "10.0.0.50"},
  {"event_type": "login_failed", "user": "admin", "ip": "10.0.0.50"},
  {"event_type": "login_failed", "user": "admin", "ip": "10.0.0.50"},
  {"event_type": "login_failed", "user": "admin", "ip": "10.0.0.50"},
  {"event_type": "login_failed", "user": "admin", "ip": "10.0.0.50"}
]
```

---

## 🧠 Detection Scores

```
Sigma Score    = 1.0
Anomaly Score  = 0.6
IOC Score      = 0.1

Final Score = 0.70 → Threshold reached
```

---

## 🔗 Correlation

Pattern detected:

* 5 failed logins within 60 seconds
  → **Brute Force Attack**

---

## 🚨 Incident Output

```json
{
  "severity": "HIGH",
  "attack_type": "Brute Force",
  "confidence": 0.82,
  "entities": ["admin", "10.0.0.50"],
  "timeline": "5 failed logins in 60 seconds"
}
```

---

## 🤖 AI Enrichment

```json
{
  "summary": "Multiple failed login attempts detected on admin account",
  "recommendations": [
    "Block source IP",
    "Reset admin password",
    "Review authentication logs"
  ]
}
```

---

# 📊 Performance (Measured)

Test environment:

* Synthetic dataset (~5,000 events)
* Single-machine execution

### Results

* **Throughput:** ~800–1200 events/sec
* **Detection latency:** ~150–300 ms per event
* **Alert delay:** < 2 seconds

⚠️ Note: No distributed scaling (single-node system)

---

# ⚙️ System Constraints

* In-memory graph limits long-term correlation
* SQLite restricts horizontal scalability
* Python layer introduces latency under heavy load

---

# 🧩 Requirement Mapping (R1–R10)

| Requirement       | Implementation            |
| ----------------- | ------------------------- |
| R1 Ingestion      | Rust event pipeline       |
| R2 Detection      | Sigma + ML + IOC          |
| R3 Anomaly        | Isolation Forest          |
| R4 Correlation    | Graph engine              |
| R5 Storage        | SQLite                    |
| R6 Reporting      | Planned                   |
| R7 Encryption     | Planned                   |
| R8 Prioritization | Composite scoring (basic) |
| R9 Dashboard      | Flutter UI                |
| R10 Remediation   | LLM-based suggestions     |

---

# 🚀 Quick Start

## 🔧 Requirements

* Rust
* Python 3.9+
* Ollama (optional for LLM)

---

## ▶️ Run

```bash
git clone https://github.com/arunethical-dotcom/Cyber-security-SOC-system-
cd Cyber-security-SOC-system-

# Rust backend
cd rust-core && cargo run

# Python API
cd ../python-ai && python main.py

# Generate events
cd ../events/generators && python generate_events.py
```

---

# 🎯 Use Cases

* SOC analyst training
* Threat detection and monitoring
* Cyber attack simulation
* AI-assisted incident response

---

# ⚠️ Limitations

* Static time windows may miss slow-moving attacks
* Linear scoring model may oversimplify threat evaluation
* Single-node architecture limits scalability
* Limited false-positive calibration

---

# 🛠️ Future Work

* Adaptive alert prioritization
* Automated remediation (SOAR integration)
* Compliance reporting engine
* Distributed pipeline (Kafka / streaming architecture)

---

# 📜 License

MIT License
