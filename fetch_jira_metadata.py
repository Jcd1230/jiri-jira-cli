import os
import re
import urllib.request
import urllib.error
import base64
import json
import sys

def get_config():
    config_path = os.path.expanduser("~/.config/jiri/config.toml")
    if not os.path.exists(config_path):
        print(f"Config not found at {config_path}")
        sys.exit(1)
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
    if data:
        req.data = json.dumps(data).encode('utf-8')
        
    try:
        with urllib.request.urlopen(req) as response:
            return json.loads(response.read().decode('utf-8'))
    except urllib.error.HTTPError as e:
        print(f"HTTP Error {e.code}: {e.read().decode('utf-8')}")
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

def main():
    username, token, base_url, project = get_config()
    print(f"Project: {project}")
    
    # 1. Fetch fields to find "Story Points"
    fields_url = f"{base_url}/rest/api/3/field"
    print("Fetching fields...")
    fields = make_request("GET", fields_url, username, token)
    story_point_fields = [f for f in fields if "story" in f["name"].lower() and "point" in f["name"].lower()]
    print("Story point fields found:")
    for f in story_point_fields:
        print(f"  {f['name']} -> ID: {f['id']}")
        
    # 2. Fetch issue types for project
    proj_url = f"{base_url}/rest/api/3/project/{project}"
    print("\nFetching project info...")
    proj_info = make_request("GET", proj_url, username, token)
    issue_types = proj_info.get("issueTypes", [])
    print("Available issue types:")
    for it in issue_types:
        print(f"  {it['name']} -> ID: {it['id']}")
        
    # 3. Create a dummy task to fetch transitions, or just query statuses
    status_url = f"{base_url}/rest/api/3/status"
    print("\nFetching global statuses...")
    statuses = make_request("GET", status_url, username, token)
    status_names = set([s['name'] for s in statuses])
    print(f"Statuses found: {', '.join(sorted(status_names)[:20])} ...")

if __name__ == "__main__":
    main()
