import os
import re
import urllib.request
import urllib.error
import base64
import json
import time
import random
import sys

def get_config():
    config_path = os.path.expanduser("~/.config/jiri/config.toml")
    with open(config_path, "r") as f:
        content = f.read()
    username = re.search(r'username\s*=\s*"([^"]+)"', content).group(1)
    token = re.search(r'token\s*=\s*"([^"]+)"', content).group(1)
    site = re.search(r'site\s*=\s*"([^"]+)"', content).group(1)
    project = re.search(r'default_project\s*=\s*"([^"]+)"', content).group(1)
    return username, token, site.rstrip('/'), project

def make_request(method, url, username, token, data=None):
    auth_str = f"{username}:{token}"
    b64_auth_str = base64.b64encode(auth_str.encode('ascii')).decode('ascii')
    
    headers = {
        'Authorization': f'Basic {b64_auth_str}',
        'Accept': 'application/json',
        'Content-Type': 'application/json'
    }
    
    req = urllib.request.Request(url, headers=headers, method=method)
    if data is not None:
        req.data = json.dumps(data).encode('utf-8')
        
    try:
        with urllib.request.urlopen(req) as response:
            body = response.read().decode('utf-8')
            return json.loads(body) if body else {}
    except urllib.error.HTTPError as e:
        print(f"HTTP Error {e.code} on {method} {url}: {e.read().decode('utf-8')}")
        return None
    except Exception as e:
        print(f"Error on {method} {url}: {e}")
        return None

def create_issue(base, auth, proj, itype, summary, parent_key=None, points=None):
    fields = {
        "project": {"key": proj},
        "summary": summary,
        "issuetype": {"name": itype}
    }
    
    if parent_key and itype == "Task":
        fields["parent"] = {"key": parent_key}
        
    if points:
        fields["labels"] = [f"story-points-{points}"]

    res = make_request("POST", f"{base}/rest/api/3/issue", auth[0], auth[1], {"fields": fields})
    return res.get("key") if res else None

def transition(base, auth, issue_key, target_name):
    # Fetch transitions available for the issue
    trans = make_request("GET", f"{base}/rest/api/3/issue/{issue_key}/transitions", auth[0], auth[1])
    if not trans or "transitions" not in trans:
        return False
        
    tid = None
    for t in trans["transitions"]:
        # Match "In Progress", "Done", etc.
        if t["name"].lower() == target_name.lower():
            tid = t["id"]
            break
            
    if tid:
        # Perform the transition
        res = make_request("POST", f"{base}/rest/api/3/issue/{issue_key}/transitions", auth[0], auth[1], {"transition": {"id": tid}})
        return res is not None
    return False

def main():
    u, t, b, p = get_config()
    auth = (u, t)
    
    print(f"Targeting Project '{p}' on site '{b}'")
    
    scenarios = ["done"] * 3 + ["mixed"] * 4 + ["todo"] * 3
    random.shuffle(scenarios)
    
    for i, scen in enumerate(scenarios, 1):
        cap_summary = f"Prototyping Epic {i} ({scen.upper()} state)"
        cap_key = create_issue(b, auth, p, "Epic", cap_summary)
        
        if not cap_key:
            print(f"Failed to create Epic for scenario {scen}. Terminating early.")
            sys.exit(1)
            
        print(f"\n[{i}/10] Created Epic: {cap_key} - Scenario: {scen}")
        
        num_tasks = random.randint(3, 5)
        for j in range(1, num_tasks + 1):
            pt = random.choice([1, 2, 3, 5])
            task_summary = f"Develop sub-component {j} for {cap_key}"
            
            t_key = create_issue(b, auth, p, "Task", task_summary, parent_key=cap_key, points=pt)
            if not t_key:
                print(f"  [!] Failed to create child task {j}.")
                continue
                
            # Apply transition scenario
            final_status = "To Do"
            if scen == "done":
                transition(b, auth, t_key, "In progress")
                transition(b, auth, t_key, "Done")
                final_status = "Done"
            elif scen == "mixed":
                state = random.choice(["To Do", "In Progress", "Done"])
                if state == "In Progress":
                    transition(b, auth, t_key, "In progress")
                elif state == "Done":
                    transition(b, auth, t_key, "In progress")
                    transition(b, auth, t_key, "Done")
                final_status = state
                
            print(f"  -> Created Task: {t_key} | Points: {pt} | Status: {final_status}")
            
if __name__ == "__main__":
    main()
