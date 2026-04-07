import fetch_jira_metadata
import pprint

username, token, base, project = fetch_jira_metadata.get_config()

# Search for issues to delete
jql = f'project = {project} AND summary ~ "Prototyping Feature"'
print("Searching for old capability tickets...")
import urllib.parse
import urllib.parse
jql_encoded = urllib.parse.quote(jql)
search_url = f"{base}/rest/api/3/search/jql"
res = fetch_jira_metadata.make_request("GET", f"{search_url}?jql={jql_encoded}&maxResults=100", username, token)

if res and "issues" in res:
    print(f"Found {len(res['issues'])} issues to delete.")
    for issue in res['issues']:
        if 'id' not in issue:
            print("Issue has no id:", issue)
            continue
        key = issue["id"]
        print(f"Deleting {key}...")
        # Since make_request uses urlopen, DELETE method needs to be handled
        try:
            req = fetch_jira_metadata.urllib.request.Request(f"{base}/rest/api/3/issue/{key}", method="DELETE")
            req.add_header('Authorization', fetch_jira_metadata.urllib.request.Request(search_url, method="POST", data=b"").headers.get('Authorization', ''))
            
            auth_str = f"{username}:{token}"
            b64_auth_str = fetch_jira_metadata.base64.b64encode(auth_str.encode('ascii')).decode('ascii')
            req.add_header('Authorization', f'Basic {b64_auth_str}')
            
            fetch_jira_metadata.urllib.request.urlopen(req)
            print(f"Deleted {key}")
        except Exception as e:
            print(f"Error deleting {key}: {e}")
