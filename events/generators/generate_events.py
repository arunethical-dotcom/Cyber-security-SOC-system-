import time
import requests
import random

URL = "http://localhost:8080/events"

users = ["admin", "user1", "guest"]
ips = ["192.168.1.10", "10.0.0.5", "172.16.0.3"]

def send_event(event):
    try:
        response = requests.post(URL, json=event)
        print(f"✅ Sent: {event} | Status: {response.status_code}")
    except requests.exceptions.ConnectionError:
        print("❌ Backend not running on port 8000")
        time.sleep(2)


def normal_traffic():
    event = {
        "event_type": "login_success",
        "entity": {"type": "user", "value": random.choice(users)},
        "metadata": {"source_ip": random.choice(ips)}
    }
    send_event(event)


def brute_force_attack():
    user = "admin"
    for _ in range(8):
        event = {
            "event_type": "login_failed",
            "entity": {"type": "user", "value": user},
            "metadata": {"source_ip": random.choice(ips)}
        }
        send_event(event)
        time.sleep(0.3)


if __name__ == "__main__":
    print("🚀 Generator started...\n")

    while True:
        mode = random.choice(["normal", "attack"])

        if mode == "normal":
            normal_traffic()
            time.sleep(1)

        else:
            print("\n⚠️ Simulating brute force attack...\n")
            brute_force_attack()
            time.sleep(3)