"""
R3 — Anomaly Detection Service
Detects anomalous incidents using Isolation Forest with online learning
"""

import numpy as np
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler
from threading import Thread
from datetime import datetime
from loguru import logger


def extract_features(event: dict) -> list:
    """
    Extract exactly 7 features from an event for anomaly detection.
    Returns features in order: hour_norm, after_hours, sigma, z_norm, cvss_norm, entity_norm, severity
    """
    features = []

    # Feature 1: hour_norm
    try:
        timestamp = event.get("timestamp", "")
        if timestamp:
            dt = datetime.fromisoformat(timestamp.replace("Z", "+00:00"))
            hour_norm = dt.hour / 23.0
        else:
            hour_norm = 0.5
    except Exception:
        hour_norm = 0.5
    features.append(hour_norm)

    # Feature 2: after_hours
    try:
        timestamp = event.get("timestamp", "")
        if timestamp:
            dt = datetime.fromisoformat(timestamp.replace("Z", "+00:00"))
            hour = dt.hour
            after_hours = 1.0 if (hour < 8 or hour > 18) else 0.0
        else:
            after_hours = 0.0
    except Exception:
        after_hours = 0.0
    features.append(after_hours)

    # Feature 3: sigma
    sigma = float(event.get("sigma_score", 0.0))
    features.append(sigma)

    # Feature 4: z_norm (clamped to [0, 10], normalized)
    z_score = float(event.get("z_score", 0.0))
    z_norm = min(z_score, 10.0) / 10.0
    features.append(z_norm)

    # Feature 5: cvss_norm
    cvss = float(event.get("cvss", 0.0))
    cvss_norm = cvss / 10.0
    features.append(cvss_norm)

    # Feature 6: entity_norm
    entities = event.get("entities", [])
    entity_count = len(entities) if isinstance(entities, list) else 0
    entity_norm = min(entity_count, 10) / 10.0
    features.append(entity_norm)

    # Feature 7: severity (mapped)
    severity_map = {"LOW": 0.1, "MEDIUM": 0.4, "HIGH": 0.7, "CRITICAL": 1.0}
    severity_str = event.get("severity", "MEDIUM").upper()
    severity = severity_map.get(severity_str, 0.4)
    features.append(severity)

    return features


def generate_seed_data(n: int = 500) -> np.ndarray:
    """
    Generate realistic normal event features for training.
    - n samples of business-hours, low-risk events
    - n//5 samples of slightly elevated but normal events
    """
    rng = np.random.default_rng(42)

    # Normal business events: low scores, within hours
    normal_samples = []
    for _ in range(n):
        hour_norm = rng.uniform(8 / 23.0, 18 / 23.0)  # Business hours
        after_hours = 0.0  # False - within business hours
        sigma = rng.uniform(0.0, 0.25)  # Low sigma
        z_norm = rng.uniform(0.0, 0.2)  # Low z
        cvss_norm = rng.uniform(0.0, 0.3)  # Low CVSS
        entity_norm = rng.uniform(0.0, 0.4)  # Few entities
        severity = 0.1  # LOW

        normal_samples.append(
            [hour_norm, after_hours, sigma, z_norm, cvss_norm, entity_norm, severity]
        )

    # Slightly elevated but still-normal events: outside hours but low scores
    elevated_samples = []
    for _ in range(n // 5):
        hour_norm = rng.uniform(0.0, 1.0)  # Any time
        after_hours = rng.choice([0.0, 1.0], p=[0.5, 0.5])  # Mixed
        sigma = rng.uniform(0.2, 0.45)  # Slightly higher
        z_norm = rng.uniform(0.0, 0.3)  # Still low
        cvss_norm = rng.uniform(0.0, 0.4)  # Still low
        entity_norm = rng.uniform(0.0, 0.5)  # Still few
        severity = 0.2  # LOW-MEDIUM

        elevated_samples.append(
            [hour_norm, after_hours, sigma, z_norm, cvss_norm, entity_norm, severity]
        )

    # Combine
    all_samples = np.array(normal_samples + elevated_samples, dtype=np.float32)
    return all_samples


class AnomalyDetector:
    """
    Detects anomalous incidents using Isolation Forest.
    Supports online learning via background retraining.
    """

    def __init__(self):
        self.model = IsolationForest(
            n_estimators=100, contamination=0.05, random_state=42, n_jobs=1
        )
        self.scaler = StandardScaler()
        self.real_events = []  # Store real event features for retraining
        self.trained = False
        self.retrain_every = 50  # Retrain after this many new events
        self._retrain_thread = None

        # Immediate training on startup
        self._initial_train()

    def _initial_train(self):
        """Train the model on seed data"""
        try:
            seed_data = generate_seed_data(500)
            self.scaler.fit(seed_data)
            self.model.fit(self.scaler.transform(seed_data))
            self.trained = True
            logger.info("R3: Model ready.")
        except Exception as e:
            logger.error(f"R3: Initial training failed: {e}")
            self.trained = False

    def _retrain(self):
        """Blend real events with shrinking synthetic data and retrain"""
        try:
            if not self.real_events:
                return

            # Calculate synthetic count
            synth_count = max(0, 500 - len(self.real_events))

            # Generate synthetic data
            if synth_count > 0:
                synthetic_data = generate_seed_data(synth_count)
            else:
                synthetic_data = np.array([])

            # Combine
            real_data = np.array(self.real_events, dtype=np.float32)
            if synth_count > 0:
                training_data = np.vstack([real_data, synthetic_data])
            else:
                training_data = real_data

            # Refit scaler and model
            self.scaler.fit(training_data)
            self.model.fit(self.scaler.transform(training_data))
            logger.debug(f"R3: Retrained with {len(self.real_events)} real + {synth_count} synthetic")
        except Exception as e:
            logger.error(f"R3: Retraining failed: {e}")

    def score(self, event: dict) -> float:
        """
        Score an event for anomaly likelihood.
        Returns 0.0 if not trained, otherwise returns float between 0 and 1.
        """
        if not self.trained:
            return 0.0

        try:
            features = extract_features(event)
            features_array = np.array([features], dtype=np.float32)
            scaled = self.scaler.transform(features_array)

            # Get raw decision function and convert to anomaly score
            raw_score = self.model.decision_function(scaled)[0]
            anomaly_score = float(np.clip(0.5 - raw_score, 0.0, 1.0))

            return round(anomaly_score, 4)
        except Exception as e:
            logger.error(f"R3: Scoring failed: {e}")
            return 0.0

    def record_event(self, event: dict):
        """
        Record a real event for online learning.
        Triggers background retraining every retrain_every events.
        """
        try:
            features = extract_features(event)
            self.real_events.append(features)

            # Trigger retraining in background if threshold reached
            if len(self.real_events) % self.retrain_every == 0:
                if self._retrain_thread and self._retrain_thread.is_alive():
                    return  # Skip if retraining already running

                self._retrain_thread = Thread(target=self._retrain, daemon=True)
                self._retrain_thread.start()
        except Exception as e:
            logger.error(f"R3: Recording event failed: {e}")


# Module-level singleton
_detector = None


def get_anomaly_detector() -> AnomalyDetector:
    """Get or create the global anomaly detector instance"""
    global _detector
    if _detector is None:
        _detector = AnomalyDetector()
    return _detector
